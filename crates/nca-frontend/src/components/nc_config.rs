use dioxus::prelude::*;
use dioxus_free_icons::{Icon, IconShape};
use dioxus_free_icons::icons::hi_outline_icons;
use dioxus_free_icons::icons::hi_solid_icons;
use daisy_rsx::*;
use dioxus_logger::tracing;
use rand::Rng;
use paspio::entropy;
use reqwest::Url;
use serde_json::json;
use web_sys::window;
use crate::{base_url, do_post};

fn generate_secure_password() -> String {
    rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(24).map(char::from)
        .collect()
}

#[derive(Clone, PartialEq, PartialOrd)]
enum PasswordStrength {
    Insecure,
    Weak,
    Strong
}

fn check_is_secure_password(pw: String) -> PasswordStrength {
    if pw.is_empty() {
        return PasswordStrength::Insecure;
    }
    match entropy(&pw) {
        e if e < 100.0 => PasswordStrength::Insecure,
        e if e < 130.0 => PasswordStrength::Weak,
        _ =>  PasswordStrength::Strong,
    }
}

// #[derive(Props, PartialEq, Clone)]
// pub struct NcConfigProps {
//     configuration_complete: Signal<bool>,
//     error: Signal<Option<String>>,
// }

#[component]
pub fn NcConfig(error: Signal<Option<String>>, configuration_complete: Signal<bool>) -> Element {
    let mut nc_admin_password = use_signal(|| "".to_string());
    let nc_domain = use_signal(|| window().unwrap().location().hostname().unwrap());


    rsx! {
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
                r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: true}),
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
                            match do_post(&request_url, payload.to_string(), None).await {
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
                                    configuration_complete.set(true);
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

#[derive(Props, Clone, PartialEq)]
pub struct PwStrengthProps {
    strength: PasswordStrength,
    insecure_icon: Element,
    weak_icon: Element,
    strong_icon: Element,
}
#[component]
pub fn PasswordStrengthIndicator(props: PwStrengthProps) -> Element {
    match props.strength {
        PasswordStrength::Insecure => props.insecure_icon,
        PasswordStrength::Weak => props.weak_icon,
        PasswordStrength::Strong => props.strong_icon,
    }
}

#[derive(Clone, PartialEq, Copy)]
struct PasswordFieldConfig {
    generator: bool,
    hide: bool,
}
#[derive(Clone, PartialEq)]
pub enum InputType {
    Text,
    Password(PasswordFieldConfig),
    Url
}

#[derive(Props, Clone, PartialEq)]
pub struct InputFieldProps {
    title: String,
    label: Option<Element>,
    value: Signal<String>,
    r#type: InputType,
    enable_copy_button: Option<bool>,
    prefix: Option<Element>
}

#[component]
pub fn InputField(mut props: InputFieldProps) -> Element {
    let password_strength = use_memo(move || check_is_secure_password(props.value.to_string()));
    let input_type_cfg = props.r#type.clone();
    let generate_password = match &input_type_cfg {
        InputType::Password(cfg) => cfg.generator,
        _ => false
    };
    use_effect(move || {
        if generate_password {
            props.value.set(generate_secure_password());
        }
    });
    let input_type = use_memo( move || {
        match props.r#type {
            InputType::Password(cfg) => {
                if cfg.hide {
                    "password"
                } else {
                    "text"
                }
            },
            InputType::Url => "url",
            InputType::Text => "text"
        }
    });

    rsx! {
        fieldset {
            class: "fieldset",
            legend {
                class: "fieldset-legend text-lg",
                "{props.title}"
            }
            label {
                class: "input flex flex-row items-center text-sx my-2",
                if let Some(prefix) = props.prefix {
                    {prefix}
                }
                input {
                    class: "grow ml-2",
                    r#type: input_type,
                    placeholder: "{props.title}",
                    value: "{props.value}",
                    oninput: move |event| props.value.set(event.value())
                },
                if generate_password {
                    button {
                        class: "btn btn-square ml-2",
                        title: "Regenerate Password",
                        onclick: move |evt: MouseEvent| {
                            props.value.set(generate_secure_password());
                            evt.prevent_default();
                        },
                        Icon {
                            class: "text-secondary",
                            icon: hi_solid_icons::HiSparkles,
                            height: 30,
                            width: 30,
                            title: "Regenerate Password"
                        },
                    }
                },
                if let Some(true) = props.enable_copy_button {
                    button {
                        class: "btn btn-square ml-2",
                        title: "Copy to Clipboard",
                        onclick: move |evt: MouseEvent| {

                            evt.prevent_default();
                        },
                        Icon {
                            class: "text-secondary",
                            icon: hi_solid_icons::HiClipboardCopy,
                            // size: 30,
                            height: 26,
                            width: 26,
                            title: "Copy to Clipboard"
                        },
                    }
                }
            //     if let(&InputType::Password(_)) = &input_type_cfg {
            //         PasswordStrengthIndicator {
            //             strength: password_strength(),
            //             insecure_icon: rsx!(Icon {
            //                 class: "ml-4 text-error",
            //                 icon: hi_solid_icons::HiExclamationCircle,
            //                 // size: 30,
            //                 height: 30,
            //                 width: 30,
            //                 fill: "currentColor"
            //             }),
            //             weak_icon: rsx!(Icon {
            //                 class: "ml-4 text-warning",
            //                 icon: hi_solid_icons::HiExclamationCircle,
            //                 // size: 30,
            //                 height: 30,
            //                 width: 30,
            //                 fill: "currentColor"
            //             }),
            //             strong_icon: rsx!(Icon {
            //                 class: "ml-4 text-success",
            //                 icon: hi_solid_icons::HiCheckCircle,
            //                 // size: 30,
            //                 height: 30,
            //                 width: 30,
            //                 fill: "currentColor"
            //             }),
            //
            //         },
            //     }
            },
            if let &InputType::Password(_) = &input_type_cfg {
                span {
                    class: "block h-1 my-2 mx-2 border-box",
                    class: if password_strength() == PasswordStrength::Insecure { "bg-error" },
                    class: if password_strength() == PasswordStrength::Weak { "bg-warning" },
                    class: if password_strength() == PasswordStrength::Strong { "bg-success" },
                    style: {
                        match password_strength() {
                            PasswordStrength::Insecure => "width: 10%;",
                            PasswordStrength::Weak => "width: 50%",
                            PasswordStrength::Strong => "",
                        }
                    }

                }
            }
            if let Some(label) = props.label {
                p {
                    class: "fieldset-label text-xs",
                    {label}
                }
            }
        }
    }
}
