use dioxus::prelude::*;
use crate::components::configure_configstep::{CfgConfigStep, ConfigStepContinueButton};
use crate::ConfigStep;

#[component]
pub fn CfgWelcome(error: Signal<Option<String>>, on_continue: EventHandler<MouseEvent>, on_validated: EventHandler<bool>) -> Element {
    on_validated(true);

    rsx! {
        CfgConfigStep {
            back_button: None,
            continue_button: rsx!(ConfigStepContinueButton{
                on_click: on_continue,
                button_text: "Continue"
            }),
            h2 {
                class: "mx-auto text-xl",
                "Welcome to Nextcloud Atomic!"
            },
        }
    }
    
}