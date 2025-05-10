use dioxus::prelude::*;
use dioxus_free_icons::{Icon, IconShape};
use dioxus_free_icons::icons::hi_outline_icons;
use dioxus_free_icons::icons::hi_solid_icons;
use daisy_rsx::*;
use dioxus_logger::tracing;
use rand::Rng;
use crate::{base_url, do_post, check_is_secure_password, generate_secure_password, ConfigStep, PasswordStrength};
use crate::components::form::{InputField, PasswordFieldConfig, InputType};
use std::borrow::Borrow;
use daisy_rsx::accordian::AccordianProps;
use dioxus::html::a::class;
use crate::components::configure_configstep::{CfgConfigStep, ConfigStepContinueButton};

#[derive(Clone, Debug, PartialEq)]
struct NcaCredentials {
    salt: String,
    primary_password: String,
    mfa_backup_codes: [String; 16],
    disk_encryption_password: String,
    backup_password: String,
}

#[cfg(feature = "mock-backend")]
fn derive_credentials_from_root_password(root_password: String) -> NcaCredentials {
    NcaCredentials {
        primary_password: root_password,
        salt: rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(12).map(char::from)
            .collect(),
        backup_password: rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(24).map(char::from)
            .collect(),
        disk_encryption_password: rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(24).map(char::from)
            .collect(),
        mfa_backup_codes: [0; 16].map(|_| rand::rng()
                .sample_iter(rand::distr::Alphanumeric)
                .take(6).map(char::from)
                .collect())
    }
}

#[cfg(not(feature = "mock-backend"))]
fn derive_credentials_from_root_password(root_password: String) -> NcaCredentials {
    return NcaCredentials{}
}

#[component]
pub fn CredentialsConfig(error: Signal<Option<String>>, on_back: EventHandler<MouseEvent>, on_continue: EventHandler<MouseEvent>) -> Element {
    let mut nca_primary_password = use_signal(|| "".to_string());
    let mut credentials: Signal<Option<NcaCredentials>> = use_signal(|| None);
    let mut step = use_signal(|| 0);
    let mut is_valid = use_signal(|| false);

    use_effect(move || {
        tracing::info!("calculating password strength ...");
        let is_strong_password = check_is_secure_password(nca_primary_password().to_string()) == PasswordStrength::Strong;
        if is_strong_password {
            credentials.set(Some(derive_credentials_from_root_password(nca_primary_password())));
        } else {
            credentials.set(None);
        }
        is_valid.set(is_strong_password)
    });
    use_effect(move || {
        match credentials() {
            Some(creds) => tracing::info!("salt: {}", creds.salt),
            None => tracing::info!("no credentials yet")
        }
    });

    rsx! {
        CfgConfigStep {
            back_button: rsx!(ConfigStepContinueButton{
                on_click: on_back,
                button_text: "Back"
            }),
            continue_button: rsx!(ConfigStepContinueButton{
                on_click: on_continue,
                button_text: "Continue",
                disabled: !is_valid()
            }),
            div {
                class: "flex-none p-2",
                div {
                    class: "pb-2",
                    Accordian {
                        title: "Primary Password",
                        name: "credential_step",
                        checked: step() == 0,
                        InputField {
                            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: true, strength_indicator: true}),
                            title: "Nextcloud Atomic Password",
                            label: rsx!(div {
                                "This password will be used to log into Nextcloud as user ",
                                span {
                                    class: "italic",
                                    "admin"
                                },
                                "."
                            }),
                            value: nca_primary_password,
                            enable_copy_button: true,
                            prefix: rsx!(
                                Icon {
                                    class: "text-secondary h-1em opacity-50",
                                    icon: hi_solid_icons::HiKey,
                                    height: 30,
                                    width: 30
                                },)
                        },
                    },
                }
                div {
                    class: "my-2",
                    Accordian {
                        title: "Time based one-time password",
                        name: "credential_step",
                        checked: step() == 1,
                        CredentialsConfigTotp {

                        }
                    }
                }
                if credentials().is_some() {
                    CredentialsConfigSummary {
                        credentials: credentials().unwrap()
                    }
                }
            }
        }
    }
}

#[component]
pub fn CredentialsConfigSummary(credentials: NcaCredentials) -> Element {
    let mut salt = use_signal(|| "".to_string());
    let mut backup_password = use_signal(|| "".to_string());
    let mut disk_encryption_password = use_signal(|| "".to_string());
    let mut mfa_backup_codes = use_signal(|| "".to_string());
    use_effect(move || {
        salt.set(credentials.salt.clone());
        backup_password.set(credentials.backup_password.clone());
        disk_encryption_password.set(credentials.disk_encryption_password.clone());
        mfa_backup_codes.set(credentials.mfa_backup_codes.join(" "));
    });

    rsx! {

        Alert {
            alert_color: Some(AlertColor::Warn),
            "The following credentials are automatically generated from your primary password. Please download or copy them and store them safely. You will need them to access your backups or your data in emergency cases."
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, strength_indicator: false}),
            disabled: true,
            title: "System Salt",
            label: rsx!(div {
                "This is an internal value used for deriving secret values from your primary password"
            }),
            value: salt,
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, strength_indicator: false}),
            disabled: true,
            title: "Disk Encryption Password",
            label: rsx!(div {
                "This password can be used to decrypt the disks of Nextcloud Atomic if automatic decryption fails"
            }),
            value: disk_encryption_password,
            enable_copy_button: true,
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, strength_indicator: false}),
            disabled: true,
            title: "Backup Password",
            label: rsx!(div {
                "This password is used to encrypt Nextcloud Atomic backups. You will need it to access your backups"
            }),
            value: backup_password,
            enable_copy_button: true,
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, strength_indicator: false}),
            disabled: true,
            title: "2nd Factor Backup Codes",
            label: rsx!(div {
                "These codes can be used if you lose access to your 2nd factor (e.g. your phone) for logging into Nextcloud Atomic"
            }),
            value: mfa_backup_codes,
            enable_copy_button: true,
        }
    }
}

#[component]
pub fn CredentialsConfigTotp() -> Element {
    let mut totp_code = use_signal(|| "".to_string());

    rsx! {
        fieldset {
            class: "fieldset",
            // legend {
            //     class: "fieldset-legend text-lg",
            //     "TOTP Setup"
            // }
            // Alert {
            //     span {
            //         "Scan the QR Code with a TOTP app (e.g. ",
            //         a {
            //             class: "text-accent",
            //             href: "https://getaegis.app",
            //             "Aegis (Android)"
            //         },
            //         " or ",
            //         a {
            //             class: "text-accent",
            //             href: "https://apps.apple.com/us/app/otp-auth/id659877384",
            //             "OTP Auth (iOS)"
            //         },
            //         ")"
            //     }
            // },
            Card {
                class: "bg-base-100 w-full center shadow-sm p-2",
                CardHeader {
                    title: "TOTP Setup",
                },
                figure {
                    class: "max-w-xs mx-auto mt-4",
                    img {
                        src: asset!("/assets/images/totp_mock_qr.png")
                    },
                },
                CardBody {
                    span {
                        "Scan the QR Code with a TOTP app for your Operating System, e.g.:",
                        ul {
                            class: "list-disc",
                            li {
                                a {
                                    class: "text-accent",
                                    href: "https://getaegis.app",
                                    "Aegis (Android)"
                                },
                            },
                            li {
                                a {
                                    class: "text-accent",
                                    href: "https://apps.apple.com/us/app/otp-auth/id659877384",
                                    "OTP Auth (iOS)"
                                },
                            },
                            li {
                                a {
                                    class: "text-accent",
                                    href: "https://apps.gnome.org/Authenticator/",
                                    "Authenticator (Linux)"
                                }
                            },
                            li {
                                a {
                                    class: "text-accent",
                                    href: "https://2fast-app.de/",
                                    "2fast (Windows)"
                                }
                            }
                        },
                        InputField {
                            r#type: InputType::Text,
                            disabled: false,
                            title: "Enter the TOTP code you generated",
                            show_title: false,
                            label: rsx!(div {
                                "Complete setting up TOTP by entering a code"
                            }),
                            value: totp_code,
                            enable_copy_button: false,
                        }
                    }
                }
            }
        }
    }
}
//
// #[derive(Props, Clone, PartialEq)]
// pub struct AccordionProps {
//     name: String,
//     title: String,
//     class: Option<String>,
//     checked: Option<bool>,
//     children: Element,
// }
//
// #[component]
// fn Accordion(props: AccordionProps) -> Element {
//
//     rsx! {
//
//         div {
//             class: String::from("collapse collapse-arrow bg-base-200") + &*props.class.unwrap_or_default(),
//             input {
//                 checked: props.checked,
//                 "type": "radio",
//                 name: props.name
//             }
//             div {
//                 class: "collapse-title text-md font-medium",
//                 "{props.title}"
//             }
//             div {
//                 class: "collapse-content  bg-base-200",
//                 {{props.children}}
//             }
//         }
//     }
// }