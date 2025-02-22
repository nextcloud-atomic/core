use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use dioxus::document::{Eval, EvalError, Script, Stylesheet};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use serde::Deserialize;
use nca_frontend::layout::{Layout, SideBar};
use nca_frontend::{assets, ServiceStatus};
use nca_frontend::components::Logs;
use web_sys::window;
use dioxus_heroicons::{Icon, mini::Shape};

use nca_system_api::types::ServiceStatus;

fn main() {
    // tracing_wasm::set_as_global_default();
    launch(App)
}



async fn run_js_eval(eval: Eval) -> Result<serde_json::Value, EvalError> {
    eval.await
}

async fn receive_js_messages(sender: tokio::sync::mpsc::Sender<String>, mut eval: Eval) {
    loop {
        let msg: String = eval.recv().await.unwrap();
        if let Err(e) = sender.send(msg).await {
            tracing::error!("Failed to forward JS message: {}", e);
            break;
        }
    }
}

#[component]
fn App() -> Element {


    rsx! {

        Stylesheet { href: asset!("assets/css/style.css") }
        Stylesheet { href: asset!("assets/css/tailwind.css") }
        // Script { src: assets::DROPIN_JS }
        Layout {    // <-- Use our layout
            title: "Nextcloud Atomic",
            headline: Some(rsx!(h1 {
                class: "text-xl",
                "Nextcloud Atomic"
            })),
            selected_item: SideBar::Activation,
            enable_sidebar: false,
            breadcrumbs: vec![],
            ServiceStatus {
                service_name: "nextcloud-all-in-one",
                on_activating: rsx! {
                    h2 {
                        class: "card-title",
                        span {
                            class: "loading loading-spinner loading-xl text-accent"
                        },
                        "Nextcloud is still starting ...",
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
                            icon: Shape::CheckCircle,
                            size: 30
                        },
                        "Nextcloud has started successfully"
                    }
                },
                on_failed: rsx! {
                    h2 {
                        class: "card-title",
                        Icon {
                            class: "text-error",
                            icon: Shape::ExclamationCircle,
                            size: 30
                        },
                        "Nextcloud failed to start"
                    }
                },
                error_action: Some(rsx! {
                    button {
                        class: "btn btn-error",
                        "Reset and restart Nextcloud"
                        b {
                            // class: "text-accent",
                            "!!!DELETES ALL DATA!!!"
                        }
                    }
                }),
                success_action: Some(rsx! {
                    button {
                        class: "btn btn-primary",
                        "Open Nextcloud"
                    }
                })

            }
            Logs {}
        }
    }
}