use std::collections::HashMap;
use std::env::current_dir;
use std::ops::Deref;
use std::thread::{current, sleep};
use std::time::Duration;
use dioxus::document::{Eval, EvalError, Script, Stylesheet};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons;
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
use nca_frontend::setup_progress_drawer::SetupProgressDrawer;

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


// rsx!(
//             ul {
//                 class: "steps min-h-24",
//                 li {
//                     class: "step step-primary",
//                     "Welcome"
//                 },
//                 li {
//                     class: "step",
//                     class: if config_status() >= ConfigStep::ConfigurePasswords { "step-primary" },
//                     "Setup Credentials"
//                 },
//                 li {
//                     class: "step",
//                     class: if config_status() >= ConfigStep::ConfigureNextcloud { "step-primary" },
//                     "Setup Nextcloud"
//                 },
//                 li {
//                     class: "step",
//                     class: if config_status() >= ConfigStep::ConfigureDisks { "step-primary" },
//                     "Setup Storage"
//                 },
//                 li {
//                     class: "step",
//                     class: if config_status() == ConfigStep::Startup { "step-primary" },
//                     "Install Nextcloud"
//                 }
//             },
// )

#[component]
fn App() -> Element {

    let mut config_status = use_signal(|| ConfigStep::Welcome);
    let mut config_is_valid = use_signal(|| false);
    let error: Signal<Option<String>> = use_signal(|| None);

    let mut config_next = move || {
        if ! &*config_is_valid.peek() {
            return
        }
        config_is_valid.set(false);
        let next_step = config_status.peek().next()
            .expect("Unexpected error: Next config step is undefined");
        config_status.set(next_step);
    };

    let mut config_back = move || {
        config_is_valid.set(false);
        let previous_step = config_status.peek().previous()
            .expect("Unexpected error: Next config step is undefined");
        config_status.set(previous_step);
    };

    rsx! {

        Stylesheet { href: asset!("assets/css/style.css") }
        Stylesheet { href: asset!("assets/css/tailwind.css") }
        // Script { src: assets::DROPIN_JS }

        div {
            class: "flex flex-row h-screen overflow-hidden",
            SetupProgressDrawer{
                current_step: config_status(),
                on_select_step: move |step: ConfigStep| if step < *config_status.peek() && step < ConfigStep::Startup {
                    config_status.set(step)
                }
            },
            main {
                id: "main-content",
                class: "flex-1 flex flex-col h-full",
                // header {
                //     class: "flex items-center p-4 border-b border-base-300",
                //     nav {
                //         aria_label: "breadcrumb",
                //         ol {
                //             class: "flex flex-wrap items-center gap-1.5 break-words text-sm sm:gap-2.5",
                //             li {
                //                 class: "ml-3 items-center gap-1.5 hidden md:block",
                //                 h1 {
                //                     class: "text-xl",
                //                     "Nextcloud Atomic"
                //                 }
                //             }
                //         }
                //     }
                // },
                section {
                    class: "flex flex-col flex-1 min-h-0",
                    if let Some(err) = error.read().deref() {
                        div {
                            role: "alert",
                            class: "alert alert-error mx-auto mt-4 mb-8 max-w-xl",
                            "{err}"
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
                                on_validated: move |is_valid: bool| config_is_valid.set(is_valid),
                                error
                            }
                        } else if config_status() == ConfigStep::ConfigurePasswords {
                            CredentialsConfig {
                                on_validated: move |is_valid: bool| config_is_valid.set(is_valid),
                                on_continue: move |_| config_next(),
                                on_back: move |_| config_back(),
                                error
                            }
                        } else if config_status() == ConfigStep::ConfigureNextcloud {
                            NextcloudConfig {
                                on_validated: move |is_valid: bool| config_is_valid.set(is_valid),
                                on_continue: move |_| config_next(),
                                on_back: move |_| config_back(),
                                error
                            }
                        } else if config_status() == ConfigStep::ConfigureDisks {
                            CfgSetupStorage {
                                on_validated: move |is_valid: bool| config_is_valid.set(is_valid),
                                on_continue: move |_| config_next(),
                                on_back: move |_| config_back(),
                                error
                            }
                        }
                    }
                }
            }
        },
    }
}

// #[derive(PartialEq, Clone, Props)]
// struct SetupProgressDrawerProps {
//     current_step: ConfigStep,
//     on_select_step: EventHandler<ConfigStep>,
//     #[props(default = String::default())]
//     class: String,
// }

// fn select_step (step: ConfigStep, current_step: ConfigStep, on_select: EventHandler<ConfigStep>) {
//     if current_step > step && current_step < ConfigStep::Startup {
//     props.on_select_step.call(step);
//     }
// }
