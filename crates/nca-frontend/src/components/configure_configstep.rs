use dioxus::prelude::*;
use crate::ConfigStep;

#[derive(Props, PartialEq, Debug, Clone)]
pub struct ConfigStepProps {
    continue_button: Element,
    back_button: Option<Element>,
    children: Element
}

#[component]
pub fn CfgConfigStep(props: ConfigStepProps) -> Element {
    rsx! {

        div {
            class: "max-h-full h-full w-1/2 max-w-4xl mx-auto my-8 center flex flex-col space-between overflow-y-auto p-4",
            div {
                class: "block grid gap-4 overflow-y-auto p-4 border-solid border-t-2 border-accent",
                { props.children }
            },
            div {
              class: "grow"
            },
            div {
                class: "flex justify-around w-full my-4",
                if let Some(back_button) = props.back_button {
                    { back_button }
                },
                { props.continue_button }
            }
        }
    }
}

#[derive(Props, PartialEq, Debug, Clone)]
pub struct ContinueButtonProps {
    #[props(default = false)]
    disabled: bool,
    #[props(default = false)]
    advancement_in_progress: bool,
    on_click: EventHandler<MouseEvent>,
    button_text: String,

}

#[component]
pub fn ConfigStepContinueButton(props: ContinueButtonProps) -> Element {

    rsx! {
        button {
            class: "btn btn-primary",
            disabled: props.disabled || props.advancement_in_progress,
            r#type: "submit",
            onclick: move |evt| props.on_click.call(evt),
            if props.advancement_in_progress {
                span {
                    class: "loading loading-spinner"
                }
            },
            { props.button_text },
        }
    }

}