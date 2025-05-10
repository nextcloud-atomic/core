use std::collections::HashMap;
use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;
use dioxus::document::{Eval, EvalError, Script, Stylesheet};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use serde::{Deserialize, Serialize};
use nca_frontend::layout::{Layout, SideBar};
use nca_frontend::{assets, base_url, NextcloudConfig, ServiceStatus};
use nca_frontend::components::{NcStartup, Logs};
use web_sys::window;
use reqwest::Client;
use serde_json::json;
use strum::IntoEnumIterator;
use strum::EnumIter;
use nca_system_api::systemd::types::ServiceStatus;
use nca_frontend::ConfigStep;
use nca_frontend::configure_credentials::{CredentialsConfig, CredentialsConfigTotp};
use nca_frontend::configure_storage::CfgSetupStorage;
use nca_frontend::configure_welcome::CfgWelcome;

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

    let mut config_status = use_signal(|| ConfigStep::Welcome);
    let mut config_is_valid = use_signal(|| false);
    let error: Signal<Option<String>> = use_signal(|| None);

    let mut config_next = move || {
        config_is_valid.set(false);
        config_status.set(config_status().next().expect("Unexpected error: Next config step is undefined"))
    };

    let mut config_back = move || {
        config_is_valid.set(false);
        config_status.set(config_status().previous().expect("Unexpected error: Next config step is undefined"))
    };

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
                class: "steps min-h-24",
                li {
                    class: "step step-primary",
                    "Welcome"
                },
                li {
                    class: "step",
                    class: if config_status() >= ConfigStep::ConfigurePasswords { "step-primary" },
                    "Setup Credentials"
                },
                li {
                    class: "step",
                    class: if config_status() >= ConfigStep::ConfigureNextcloud { "step-primary" },
                    "Setup Nextcloud"
                },
                li {
                    class: "step",
                    class: if config_status() >= ConfigStep::ConfigureDisks { "step-primary" },
                    "Setup Storage"
                },
                li {
                    class: "step",
                    class: if config_status() == ConfigStep::Startup { "step-primary" },
                    "Install Nextcloud"
                }
            },

            if config_status() == ConfigStep::Startup {
                NcStartup {
                    error
                }
            } else {
                if config_status() == ConfigStep::Welcome {
                    CfgWelcome {
                        on_continue: move |_| config_next(),
                        error
                    }
                } else if config_status() == ConfigStep::ConfigurePasswords {
                    CredentialsConfig {
                        on_continue: move |_| config_next(),
                        on_back: move |_| config_back(),
                        error
                    }
                } else if config_status() == ConfigStep::ConfigureNextcloud {
                    NextcloudConfig {
                        on_continue: move |_| config_next(),
                        on_back: move |_| config_back(),
                        error
                    }
                } else if config_status() == ConfigStep::ConfigureDisks {
                    CfgSetupStorage {
                        on_continue: move |_| config_next(),
                        on_back: move |_| config_back(),
                        error
                    }
                }
            }
        },
    }
}
