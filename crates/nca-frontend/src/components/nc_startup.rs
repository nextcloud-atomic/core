use dioxus::prelude::*;
use dioxus_free_icons::{Icon,icons::hi_solid_icons};
use dioxus_logger::tracing;
use reqwest::Client;
use serde_json::json;
use web_sys::window;
use crate::{base_url, do_post, Logs, ServiceStatus};

#[component]
pub fn NcStartup(error: Signal<Option<String>>) -> Element{

    rsx! {
        ServiceStatus {
            service_name: "nextcloud-all-in-one",
            on_activating: rsx! {
                h2 {
                    class: "card-title",
                    span {
                        class: "loading loading-spinner loading-xl text-accent"
                    },
                    "Nextcloud is still being configured ...",
                }
                p {
                    "This might take a while"
                }
            },
            on_active: rsx! {
                h2 {
                    class: "card-title",
                    Icon {
                        class: "text-accent",
                        icon: hi_solid_icons::HiCheckCircle,
                        // size: 30
                        height: 30,
                        width: 30
                    },
                    "Nextcloud was installed successfully"
                }
            },
            on_failed: rsx! {
                h2 {
                    class: "card-title",
                    Icon {
                        class: "text-error",
                        icon: hi_solid_icons::HiExclamationCircle,
                        height: 30,
                        width: 30
                    },
                    "Nextcloud failed to start"
                }
            },
            error_action: Some(rsx! {
                button {
                    onclick: move |_| error.set(Some("coming soon ...".to_string())),
                    class: "btn btn-error",
                    "Reset and restart Nextcloud"
                    b {
                        "!!!DELETES ALL DATA!!!"
                    }
                }
            }),
            success_action: Some(rsx! {
                button {
                    class: "btn btn-primary",
                    onclick: move |_| async move {
                        let request_url = format!("{}/api/setup/caddy/endpoint/enable/nextcloud", base_url());
                        tracing::info!("requesting {}", request_url);
                        let domain = window().unwrap().location().hostname().unwrap();
                        match do_post(&request_url, json!({"trusted_url": domain}).to_string(), None).await {
                            Err(e) => {
                                tracing::error!("ERROR: Activating nextcloud failed: {e:?}!")
                            },
                            Ok(response) => {
                                if response.status().is_success() {
                                    window().unwrap().location().reload().unwrap();
                                } else {
                                    let msg = format!("ERROR: Activating nextcloud failed (http status: {}): {}",
                                        response.status().as_str(),
                                        response.text().await.unwrap_or(String::from("no response body received")));
                                    tracing::error!("{}", msg);
                                    error.set(Some(msg));
                                }
                            }
                        }
                    },
                    "Go to Nextcloud"
                }
            })
        
        }
        Logs {}
    }
}