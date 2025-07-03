use std::time::Duration;
use dioxus::prelude::*;
use dioxus_free_icons::{Icon,icons::hi_solid_icons};
use dioxus_logger::tracing;
use http::StatusCode;
use reqwest::{Client, Url};
use serde_json::json;
use web_sys::window;
use crate::{base_url, do_get, do_post, HttpResponse, Logs, MockResponse, ServiceStatus};

#[cfg(not(feature = "mock-backend"))]
async fn perform_nextcloud_hard_reset() -> Result<HttpResponse, String> {
    let request_url = format!("{}/api/setup/nextcloud/hard-reset", base_url());
    do_get(&request_url, None).await.map_err(|e| e.to_string())
}

#[cfg(feature = "mock-backend")]
async fn perform_nextcloud_hard_reset() -> Result<HttpResponse, String> {
    Ok(MockResponse{
        status: StatusCode::ACCEPTED,
        body: "success".to_string(),
        url: Url::parse(format!("{}/api/setup/nextcloud/hard-reset", base_url()).as_str()).unwrap()
    }.into())
}


#[component]
pub fn NcStartup(error: Signal<Option<String>>) -> Element{

    let mut goto_nextcloud_in_progress = use_signal(|| false);

    rsx! {
        ServiceStatus {
            service_name: "nextcloud-all-in-one",
            success_action_in_progress: goto_nextcloud_in_progress(),
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
                    onclick: move |evt| async move {
                        if let Err(e) = perform_nextcloud_hard_reset().await {
                            error.set(Some(e.to_string()));
                        }
                    },
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
                    class: if goto_nextcloud_in_progress() { "btn-disabled" },
                    onclick: move |_| async move {
                        goto_nextcloud_in_progress.set(true);
                        let request_url = format!("{}/api/setup/caddy/endpoint/enable/nextcloud", base_url());
                        tracing::info!("requesting {}", request_url);
                        let domain = window().unwrap().location().hostname().unwrap();
                        match do_post(&request_url, json!({"trusted_url": domain}).to_string(), None).await {
                            Err(e) => {
                                tracing::error!("ERROR: Activating nextcloud failed: {e:?}!");
                                error.set(Some(format!("Activating nextcloud failed: {e:?}")));
                            },
                            Ok(response) => {
                                if !response.status().is_success() {
                                    let msg = format!("ERROR: Activating nextcloud failed (http status: {}): {}",
                                        response.status().as_str(),
                                        response.text().await.unwrap_or(String::from("no response body received")));
                                    tracing::error!("{}", msg);
                                    error.set(Some(msg));
                                    goto_nextcloud_in_progress.set(false);
                                    return;
                                }
                            }
                        };
                        let request_url = format!("{}/login", base_url());
                        let max = 300;
                        for i in {1..max} {
                            match do_get(&request_url, None).await {
                                Err(e) => {
                                    async_std::task::sleep(Duration::from_secs(1)).await;
                                    if i == max {
                                        let msg = "Nextcloud is still not reachable - something seems to have gone wrong".to_string();
                                        tracing::error!(msg);
                                        error.set(Some(msg));
                                        goto_nextcloud_in_progress.set(false);
                                        return;
                                    }
                                },
                                Ok(_) => {
                                    break;
                                }
                            };
                        }

                        if let Err(e) = window().unwrap().location().replace(format!("{}/login", base_url()).as_ref()) {
                            let msg = "Failed to redirect you to Nextcloud. Please reload manually".to_string();
                            tracing::error!(msg);
                            error.set(Some(msg));
                        }
                        goto_nextcloud_in_progress.set(false);
                    },
                    if goto_nextcloud_in_progress() {
                        span {
                            class: "loading loading-spinner"
                        }
                    },
                    "Go to Nextcloud",
                }
            })
        
        }
        Logs {}
    }
}