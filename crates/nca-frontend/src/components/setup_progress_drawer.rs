use std::rc::Rc;
use std::sync::Mutex;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons;
use dioxus_logger::tracing;
use crate::{ConfigStep, ConfigStepStatus, ConfigStepWithStatus, GenericStep, ServicesConfig, StepStatus};
use crate::configure_credentials::CredentialsConfig;


#[component]
pub fn SetupProgressDrawer(steps: Vec<ConfigStepWithStatus>, current_step_id: usize, on_select_step: EventHandler<usize>) -> Element {

    rsx!{
        div {
            class: "max-w-80 drawer lg:drawer-open",
            input {
                r#id: "setup-progress-steps-drawer",
                r#type: "checkbox",
                class: "drawer-toggle"
            },
            div {
                class: "drawer-content flex flex-col items-center justify-center",
                label {
                    r#for: "setup-progress-steps-drawer",
                    class: "btn btn-primary drawer-button lg:hidden",
                    "Show Setup Steps"
                }
            },
            div {
                class: "drawer-side",
                label {
                    r#for: "setup-progress-steps-drawer",
                    aria_label: "hide setup steps",
                    class: "drawer-overlay"
                },
                ol {
                    class: "menu list-decimal list-inside bg-base-200 text-base-content min-h-full w-80 p-4",
                    li {
                        class: "menu-title text-xl hover:bg-inherit",
                        h2 { "Setup Nextcloud Atomic" }
                    }
                    for (idx, ConfigStepWithStatus{step, status}) in steps.iter().enumerate() {
                        if let ConfigStep::Welcome = step {
                            SetupProgressStep{
                                title: "Welcome",
                                idx: 1,
                                is_active: current_step_id == idx,
                                is_complete: status.completed,
                                on_select: move || on_select_step.call(idx)
                            },
                        } else if let ConfigStep::Credentials = step {
                            SetupProgressStep{
                                title: "Setup Credentials",
                                idx: 2,
                                is_active: current_step_id == idx,
                                is_complete: status.completed,
                                on_select: move || on_select_step.call(idx)
                            },
                        } else  if let ConfigStep::Nextcloud = step {
                            SetupProgressStep{
                                title: "Setup Nextcloud",
                                idx: 3,
                                is_active: current_step_id == idx,
                                is_complete: status.completed,
                                on_select: move || on_select_step.call(idx)
                            },
                        } else if let ConfigStep::Disks = step {
                        
                            SetupProgressStep{
                                title: "Setup Storage",
                                idx: 4,
                                is_active: current_step_id == idx,
                                is_complete: status.completed,
                                on_select: move || on_select_step.call(idx)
                            },
                        } else if let ConfigStep::Startup = step {
                            SetupProgressStep{
                                title: "Start Services",
                                idx: 5,
                                is_active: current_step_id == idx,
                                is_complete: status.completed,
                                on_select: move || on_select_step.call(idx)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SetupProgressStep(title: String, is_active: bool, is_complete: bool, idx: usize, on_select: EventHandler) -> Element {
    rsx!{
        li {
            a {
                class: if is_active { "bg-primary text-primary-content hover:bg-primary/70 hover:text-primary-content" },
                class: if !is_active && !is_complete { "opacity-50 cursor-text hover:bg-inherit active:bg-inherit" },
                onclick: move |_| on_select(()),
                div {
                    class: "text-xl font-thin opacity-80 flex-none",
                    {format!("{:0>2}", idx)}
                },
                { title },
                if is_complete {
                    Icon {
                        class: "text-success",
                        icon: hi_solid_icons::HiCheckCircle
                    }
                } else if is_active {
                    Icon {
                        class: "text-info",
                        icon: hi_solid_icons::HiCog
                    }
                }
            }
        }
    }
}