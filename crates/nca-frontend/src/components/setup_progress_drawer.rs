use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons;
use crate::ConfigStep;

#[component]
pub fn SetupProgressDrawer(current_step: ConfigStep, on_select_step: EventHandler<ConfigStep>) -> Element {

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
                    SetupProgressStep{
                        title: "Welcome",
                        idx: 1,
                        is_active: current_step == ConfigStep::Welcome,
                        is_complete: current_step > ConfigStep::Welcome,

                        on_select: move || on_select_step.call(ConfigStep::Welcome)
                    },
                    SetupProgressStep{
                        title: "Setup Credentials",
                        idx: 2,
                        is_active: current_step == ConfigStep::ConfigurePasswords,
                        is_complete: current_step > ConfigStep::ConfigurePasswords,
                        // on_select: || update_current_step(ConfigStep::ConfigurePasswords)
                        on_select: move || on_select_step.call(ConfigStep::ConfigurePasswords)
                    },
                    SetupProgressStep{
                        title: "Setup Nextcloud",
                        idx: 3,
                        is_active: current_step == ConfigStep::ConfigureNextcloud,
                        is_complete: current_step > ConfigStep::ConfigureNextcloud,
                        on_select: move || on_select_step.call(ConfigStep::ConfigureNextcloud)
                    },
                    SetupProgressStep{
                        title: "Setup Storage",
                        idx: 4,
                        is_active: current_step == ConfigStep::ConfigureDisks,
                        is_complete: current_step > ConfigStep::ConfigureDisks,
                        on_select: move || on_select_step.call(ConfigStep::ConfigureDisks)
                    },
                    SetupProgressStep{
                        title: "Install Nextcloud",
                        idx: 5,
                        is_active: current_step == ConfigStep::Startup,
                        is_complete: current_step > ConfigStep::Startup,
                        on_select: move || {}
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
                {{ title }},
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