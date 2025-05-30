use std::cell::RefCell;
use std::collections::HashMap;
use std::env::current_dir;
use std::ops::Deref;
use std::rc::Rc;
use std::thread::{current, sleep};
use std::time::Duration;
use dioxus::document::{Eval, EvalError, Script, Stylesheet};
use dioxus::html::mover::accent;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons;
use dioxus_logger::tracing;
use serde::{Deserialize, Serialize};
use nca_frontend::layout::{Layout, SideBar};
use nca_frontend::{assets, base_url, ConfigStep, ConfigStepStatus, ConfigStepWithStatus, GenericStep, NextcloudConfig, ServiceStatus};
use nca_frontend::components::{NcStartup, Logs};
use web_sys::window;
use reqwest::Client;
use serde_json::json;
use strum::IntoEnumIterator;
use strum::EnumIter;
use nca_system_api::systemd::types::ServiceStatus;
use nca_frontend::{StepStatus};
// use nca_frontend::ConfigStep::*;
use nca_frontend::configure_credentials::{CfgCredentials, CredentialsConfig, CredentialsConfigTotp};
use nca_frontend::configure_storage::CfgSetupStorage;
use nca_frontend::configure_welcome::CfgWelcome;
use nca_frontend::configure_nextcloud::CfgNextcloud;
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

#[component]
fn App() -> Element {

    // Application State
    let creds_config = Rc::new(use_signal(|| CredentialsConfig::new()));
    let creds_status = use_signal(|| ConfigStepStatus::new());
    let nc_config = use_signal(|| NextcloudConfig::new());
    let nc_status = use_signal(|| ConfigStepStatus::new());
    let disks_status = use_signal(|| ConfigStepStatus::new());
    let startup_status = use_signal(|| ConfigStepStatus::new());

    let nc_admin_pw = {
        let cfg = creds_config.clone();
        use_memo(move || cfg().nc_admin_password.unwrap_or(String::default()))
    };

    let steps: Vec<ConfigStep> = vec![
        ConfigStep::Welcome,
        ConfigStep::Credentials,
        ConfigStep::Nextcloud,
        ConfigStep::Disks,
        ConfigStep::Startup
    ];

    let steps_with_status = use_memo(move || vec![
        ConfigStepWithStatus {
            step: ConfigStep::Welcome,
            status: ConfigStepStatus { visited: true, valid: true, completed: true }
        },
        ConfigStepWithStatus {
            step: ConfigStep::Credentials,
            status: creds_status()
        },
        ConfigStepWithStatus {
            step: ConfigStep::Nextcloud,
            status: nc_status()
        },
        ConfigStepWithStatus {
            step: ConfigStep::Disks,
            status: disks_status()
        },
        ConfigStepWithStatus {
            step: ConfigStep::Startup,
            status: startup_status()
        }
    ]);


    let mut active_step_id = use_signal(|| 0);
    let active_step = {
        let steps_cp = steps.clone();
        use_memo(move || steps_cp[active_step_id()].clone())
    };


    // let is_active_step_completed = use_memo(move || active_step().completed());
    let error: Signal<Option<String>> = use_signal(|| None);

    let can_advance_to_step = {
        let steps_len = steps.len();
        move |step_id, all_steps: Vec<ConfigStepWithStatus>| {
            if step_id >= steps_len {
                return false
            }
            if step_id == 0 {
                return all_steps[0].status.visited
            }
            let step: &ConfigStepWithStatus = &all_steps[step_id];
            let previous_step: &ConfigStepWithStatus = &all_steps[step_id - 1];
            step.status.visited
                || (step_id - 1 < steps_len - 1)
                && previous_step.status.completed
                && all_steps[0..(step_id - 1)].iter().all(|step: &ConfigStepWithStatus| {
                step.status.visited || step.status.valid
            })
        }
    };

    let can_advance_active_step = use_memo(move || {
        let step_id = active_step_id();
        let all_steps = steps_with_status();
        can_advance_to_step(step_id + 1, all_steps)
    });

    let mut advance_step = move || {
        let all_steps = steps_with_status.peek();
        if ! can_advance_to_step(*active_step_id.peek() + 1, all_steps.clone()) {
            tracing::info!("Can't advance step");
            return
        }
        let newval = {
            1 + *active_step_id.peek()
        };
        active_step_id.set(newval);
    };

    let mut revert_step = move || {
        if **&active_step_id.peek() == 0 {
            return ;
        }
        active_step_id.set(active_step_id() - 1)
    };

    let mut set_active_step = {
        let step_len = steps.len();
        move |step_id: usize| {
            let all_steps = steps_with_status.peek().clone();
            if step_id >= 0 && step_id < step_len && can_advance_to_step(step_id, all_steps) {
                active_step_id.set(step_id);
            }
        }
    };

    rsx! {

        Stylesheet { href: asset!("assets/css/style.css") }
        Stylesheet { href: asset!("assets/css/tailwind.css") }
        // Script { src: assets::DROPIN_JS }

        div {
            class: "flex flex-row h-screen overflow-hidden",
            SetupProgressDrawer{
                steps: steps_with_status(),
                current_step_id: active_step_id(),
                on_select_step: move |step_id| {
                    set_active_step(step_id);
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
                    match active_step() {
                        ConfigStep::Welcome => rsx!(
                            CfgWelcome {
                                on_continue: move |_| advance_step(),
                                error
                            }
                        ),
                        ConfigStep::Credentials => rsx!(
                            CfgCredentials {
                                on_continue: move |_| advance_step(),
                                on_back: move |_| revert_step(),
                                error,
                                config: *creds_config,
                                status: creds_status
                            }
                        ),
                        ConfigStep::Nextcloud => rsx!(
                            CfgNextcloud {
                                on_continue: move |_| advance_step(),
                                on_back: move |_| revert_step(),
                                error,
                                config: nc_config,
                                status: nc_status,
                                nc_admin_password: nc_admin_pw()
                            }
                        ),
                        ConfigStep::Disks => rsx!(
                            CfgSetupStorage {
                                on_continue: move |_| advance_step(),
                                on_back: move |_| revert_step(),
                                error,
                                status: disks_status
                            }
                        ),
                        ConfigStep::Startup => rsx!(
                            NcStartup {
                                error
                            }
                        )
                    }
                }
            }
        },
    }
}
