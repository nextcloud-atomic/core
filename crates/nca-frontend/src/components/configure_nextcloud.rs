use dioxus::prelude::*;
use daisy_rsx::*;
use dioxus_logger::tracing;
use reqwest::{Body, Response, ResponseBuilderExt, Url};
use serde_json::json;
use web_sys::window;
use dioxus_free_icons::{Icon, IconShape};
use dioxus_free_icons::icons::hi_outline_icons;
use dioxus_free_icons::icons::hi_solid_icons;
use http::StatusCode;
use crate::components::form::{InputField, InputType, PasswordFieldConfig};
use crate::{base_url, check_is_secure_password, do_post, ConfigStep, HttpResponse, MockResponse, PasswordStrength};
use crate::components::configure_configstep::{CfgConfigStep, ConfigStepContinueButton};
// #[derive(Props, PartialEq, Clone)]
// pub struct NcConfigProps {
//     configuration_complete: Signal<bool>,
//     error: Signal<Option<String>>,
// }

#[cfg(not(feature = "mock-backend"))]
async fn configure_nextcloud_credentials(nc_domain: Option<String>, nc_admin_password: String) -> Result<HttpResponse, reqwest::Error> {
    let request_url = format!("{}/api/setup/configure", base_url());
    let payload = json!({
        "nextcloud_domain": nc_domain,
        "nextcloud_password": nc_admin_password
    });
    do_post(&request_url, payload.to_string(), None).await
}

#[cfg(feature = "mock-backend")]
async fn configure_nextcloud_credentials(nc_domain: Option<String>, nc_admin_password: String) -> Result<HttpResponse, reqwest::Error> {
    let request_url = format!("{}/api/setup/configure", base_url());
    
    let resp = MockResponse{
        body: "".to_string(),
        url: Url::parse(&request_url).unwrap(),
        status: StatusCode::OK,
    };
    Ok(resp.into())
}

#[component]
pub fn NextcloudConfig(error: Signal<Option<String>>, on_back: EventHandler<MouseEvent>, on_continue: EventHandler<MouseEvent>, on_validated: EventHandler<bool>) -> Element {
    let mut nc_admin_password = use_signal(|| "".to_string());
    let nc_domain = use_signal(|| window().unwrap().location().hostname().unwrap());
    let mut is_valid = use_signal(|| false);
    let nc_admin_password_strength = use_signal(|| check_is_secure_password(nc_admin_password()));
    let propagate_validation = use_effect(move || on_validated(is_valid()));


    rsx! {
        CfgConfigStep {
            back_button: rsx!(ConfigStepContinueButton{
                on_click: on_back,
                button_text: "Back"
            }),
            continue_button: rsx!(ConfigStepContinueButton{
                on_click: on_continue,
                button_text: "Continue",
                disabled: !is_valid()
            }),
            div {
                class: "block max-w-4xl mx-auto grid center gap-4",
    
                InputField {
                    r#type: InputType::Text,
                    title: "Nextcloud URL",
                    label: rsx!(div {
                        "The domain where Nextcloud will be accessible.",
                    }),
                    value: nc_domain,
                    enable_copy_button: false,
                    prefix: rsx!(b {"https://"})
                },
                InputField {
                    r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: true, password_strength: Some(nc_admin_password_strength())}),
                    title: "Nextcloud admin password",
                    label: rsx!(div {
                        "This password will be used to log into Nextcloud as user ",
                        span {
                            class: "italic",
                            "admin"
                        },
                        "."
                    }),
                    value: nc_admin_password,
                    enable_copy_button: true,
                    prefix: rsx!(
                        Icon {
                            class: "text-secondary h-1em opacity-50",
                            icon: hi_solid_icons::HiKey,
                            height: 30,
                            width: 30
                        },)
                },
    
                button {
                    class: "btn btn-primary",
                    r#type: "submit",
                    onclick:  move |evt: Event<MouseData>| {
                        evt.prevent_default();
                        evt.stop_propagation();
                        async move {
                            if check_is_secure_password(nc_admin_password.peek().to_string()) != PasswordStrength::Strong {
                                error.set(Some("Error: The configured password is insecure!".to_string()));
                                return;
                            }
                            if let Ok(nc_url) = Url::parse(&format!("https://{}/", nc_domain.peek().to_string())) {
                                let request_url = format!("{}/api/setup/configure", base_url());
                                let payload = json!({
                                        "nextcloud_domain": nc_url.host_str(),
                                        "nextcloud_password": nc_admin_password.peek().to_string()
                                    });
                                match configure_nextcloud_credentials(
                                    nc_url.host_str().and_then(|s| Some(s.to_string())), 
                                    nc_admin_password.peek().to_string()).await {
                                    Err(e) => {
                                        tracing::error!("ERROR: Configuring Nextcloud Atomic failed: {e:?}");
                                        error.set(Some(format!("Error: Configurating Nextcloud Atomic failed; {}", e)));
                                    },
                                    Ok(response) => {
                                        if !response.status().is_success() {
                                            let msg = format!("ERROR: Configuring Nextcloud Atomic failed (http status: {}): {}",
                                                response.status().as_str(),
                                                response.text().await.unwrap_or(String::from("no response body received")));
                                            tracing::error!("{}", msg);
                                            error.set(Some(msg));
                                            return;
                                        }
                                        tracing::info!("configuration completed successfully");
                                        is_valid.set(true)
                                    }
                                }
                            } else {
                                error.set(Some(format!("Error: '{}' is not a valid domain", nc_domain.peek().to_string())));
                            }
                        }
                    },
                    "Apply",
                    Icon {
                        icon: hi_solid_icons::HiArrowRight,
                        // size: 30,
                        height: 30,
                        width: 30
                    },
                }
            }
        }
    }
}
