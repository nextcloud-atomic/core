use std::collections::HashMap;
use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;
use dioxus::document::{Eval, EvalError, Script, Stylesheet};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use serde::Deserialize;
use nca_frontend::layout::{Layout, SideBar};
use nca_frontend::{assets, base_url, NcConfig, ServiceStatus};
use nca_frontend::components::{NcStartup, Logs};
use web_sys::window;
use reqwest::Client;
use serde_json::json;
use nca_system_api::systemd::types::ServiceStatus;

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

    let mut configuration_complete = use_signal(|| false);
    let error: Signal<Option<String>> = use_signal(|| None);

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
            if let Some(err) = error.read().deref() {
                div {
                    role: "alert",
                    class: "alert alert-error mx-auto mt-4 mb-8 max-w-xl",
                    "{err}"
                }
            },
            ul {
                class: "steps",
                li {
                    class: "step step-primary",
                    "Configure"
                },
                li {
                    class: "step",
                    class: if configuration_complete() { "step-primary" },
                    "Install Nextcloud"
                }
            },
            if configuration_complete() {
                NcStartup {
                    error
                }
            } else {
                NcConfig {
                    configuration_complete,
                    error
                }
            }
        },
    }
}
