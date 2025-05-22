use std::fmt::{Display, Formatter};
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, IconShape};
use dioxus_free_icons::icons::hi_outline_icons;
use dioxus_free_icons::icons::hi_solid_icons;
use crate::{check_is_secure_password, generate_secure_password, PasswordStrength};

#[component]
pub fn InputField(mut props: InputFieldProps) -> Element {
    let input_type_cfg = props.r#type.clone();
    let generate_password = match &input_type_cfg {
        InputType::Password(cfg) => cfg.generator,
        _ => false
    };
    use_effect(move || {
        if generate_password {
            props.value.set(generate_secure_password());
        }
    });
    let input_type = use_memo( move || {
        match props.r#type {
            InputType::Password(cfg) => {
                if cfg.hide {
                    "password"
                } else {
                    "text"
                }
            },
            InputType::Url => "url",
            InputType::Text => "text"
        }
    });

    rsx! {
        fieldset {
            class: "fieldset",
            if props.show_title.unwrap_or(true) {
                legend {
                    class: "fieldset-legend text-lg",
                    "{props.title}"
                }
            }
            label {
                class: "input flex flex-row items-center text-sx my-2",
                if let Some(prefix) = props.prefix {
                    {prefix}
                }
                input {
                    class: "grow ml-2",
                    r#type: input_type,
                    disabled: props.disabled,
                    placeholder: "{props.title}",
                    value: "{props.value}",
                    oninput: move |event| {
                        props.value.set(event.value())
                    }
                },
                if generate_password {
                    button {
                        class: "btn btn-square ml-2",
                        title: "Regenerate Password",
                        onclick: move |evt: MouseEvent| {
                            props.value.set(generate_secure_password());
                            evt.prevent_default();
                        },
                        Icon {
                            class: "text-secondary",
                            icon: hi_solid_icons::HiSparkles,
                            height: 30,
                            width: 30,
                            title: "Regenerate Password"
                        },
                    }
                },
                if let Some(true) = props.enable_copy_button {
                    button {
                        class: "btn btn-square ml-2",
                        title: "Copy to Clipboard",
                        onclick: move |evt: MouseEvent| {

                            evt.prevent_default();
                        },
                        Icon {
                            class: "text-secondary",
                            icon: hi_solid_icons::HiClipboardCopy,
                            // size: 30,
                            height: 26,
                            width: 26,
                            title: "Copy to Clipboard"
                        },
                    }
                }
            //     if let(&InputType::Password(_)) = &input_type_cfg {
            //         PasswordStrengthIndicator {
            //             strength: password_strength(),
            //             insecure_icon: rsx!(Icon {
            //                 class: "ml-4 text-error",
            //                 icon: hi_solid_icons::HiExclamationCircle,
            //                 // size: 30,
            //                 height: 30,
            //                 width: 30,
            //                 fill: "currentColor"
            //             }),
            //             weak_icon: rsx!(Icon {
            //                 class: "ml-4 text-warning",
            //                 icon: hi_solid_icons::HiExclamationCircle,
            //                 // size: 30,
            //                 height: 30,
            //                 width: 30,
            //                 fill: "currentColor"
            //             }),
            //             strong_icon: rsx!(Icon {
            //                 class: "ml-4 text-success",
            //                 icon: hi_solid_icons::HiCheckCircle,
            //                 // size: 30,
            //                 height: 30,
            //                 width: 30,
            //                 fill: "currentColor"
            //             }),
            //
            //         },
            //     }
            },
            if let &InputType::Password(cfg) = &input_type_cfg {
                match cfg.password_strength {
                    Some(pw_strength) =>  {
                        rsx!{
                            span {
                                class: "block h-1 my-2 mx-2 border-box",
                                class: if pw_strength == PasswordStrength::Insecure { "bg-error" },
                                class: if pw_strength == PasswordStrength::Weak { "bg-warning" },
                                class: if pw_strength == PasswordStrength::Strong { "bg-success" },
                                style: {
                                    match pw_strength {
                                        PasswordStrength::Insecure => "width: 10%;",
                                        PasswordStrength::Weak => "width: 50%",
                                        PasswordStrength::Strong => "",
                                    }
                                }
                            }
                        }
                    },
                    None => rsx!()
                }
            }
            if let Some(label) = props.label {
                p {
                    class: "fieldset-label text-xs",
                    {label}
                }
            }
        }
    }
}


#[derive(Props, Clone, PartialEq)]
pub struct PwStrengthProps {
    strength: PasswordStrength,
    insecure_icon: Element,
    weak_icon: Element,
    strong_icon: Element,
}
#[component]
pub fn PasswordStrengthIndicator(props: PwStrengthProps) -> Element {
    match props.strength {
        PasswordStrength::Insecure => props.insecure_icon,
        PasswordStrength::Weak => props.weak_icon,
        PasswordStrength::Strong => props.strong_icon,
    }
}

#[derive(Clone, PartialEq, Copy)]
pub struct PasswordFieldConfig {
    pub(crate) generator: bool,
    pub(crate) hide: bool,
    pub(crate) password_strength: Option<PasswordStrength>,
}
#[derive(Clone, PartialEq)]
pub enum InputType {
    Text,
    Password(PasswordFieldConfig),
    Url
}

#[derive(Props, Clone, PartialEq)]
pub struct InputFieldProps {
    title: String,
    label: Option<Element>,
    value: Signal<String>,
    r#type: InputType,
    show_title: Option<bool>,
    enable_copy_button: Option<bool>,
    prefix: Option<Element>,
    disabled: Option<bool>
}
