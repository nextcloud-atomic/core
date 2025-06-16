use std::cell::RefCell;
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, IconShape};
use dioxus_free_icons::icons::hi_outline_icons;
use dioxus_free_icons::icons::hi_solid_icons;
use daisy_rsx::*;
use dioxus_logger::tracing;
use rand::Rng;
use nca_api_model::setup::CredentialsConfig as ApiCredentialsConfig;
use crate::{base_url, do_post, check_is_secure_password, generate_secure_password, PasswordStrength, StepStatus, ConfigStepStatus};
use crate::components::form::{InputField, PasswordFieldConfig, InputType};
use std::rc::Rc;
use daisy_rsx::accordian::AccordianProps;
use dioxus::html::a::class;
use serde::Serialize;
use nca_api_model::setup;
use crate::components::accordion::Accordion;
use crate::components::configure_configstep::{CfgConfigStep, ConfigStepContinueButton};
use crate::configure_credentials_backup::{ConfigureCredentialsBackup, ConfigureCredentialsConfirm};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize)]
pub struct CredentialsConfig {
    pub salt: Option<String>,
    pub primary_password: Option<String>,
    pub nc_admin_password: Option<String>,
    // pub mfa_backup_codes: Option<[String; 16]>,
    pub disk_encryption_password: Option<String>,
    pub backup_encryption_password: Option<String>,
    pub backup_id: Option<String>,
}

impl CredentialsConfig {
    
    pub fn new() -> Self {
        CredentialsConfig {
            primary_password: None,
            nc_admin_password: None,
            backup_id: None,
            disk_encryption_password: None,
            backup_encryption_password: None,
            // mfa_backup_codes: None,
            salt: None
        }
    }

}

#[cfg(feature = "mock-backend")]
async fn derive_credentials_from_primary_password(primary_password: String, nc_admin_password: String) -> Result<CredentialsConfig, String> {
    Ok(CredentialsConfig {
        primary_password: self.primary_password,
        nc_admin_password: Some(rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(24).map(char::from)
            .collect()),
        salt : Some(rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(12).map(char::from)
            .collect()),
        backup_id : Some(rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(12).map(char::from)
            .collect()),
        disk_encryption_password: Some(rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(24).map(char::from)
            .collect()),
        mfa_backup_codes : Some([0; 16].map( | _ | rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(6).map(char::from)
            .collect()))
    })

}

#[cfg(not(feature = "mock-backend"))]
async fn derive_credentials_from_primary_password(primary_password: String, nc_admin_password: String) -> Result<CredentialsConfig, String> {
    let request_url = format!("{}/api/setup/credentials", base_url());
    let payload =  serde_json::to_string(&setup::CredentialsInitRequest{
        primary_password: primary_password.clone(),
        nextcloud_admin_password: nc_admin_password.clone()
    }).map_err(|e| e.to_string())?;
    let result = do_post(&request_url, payload, None).await.map_err(|e| e.to_string())?;

    let credentials: ApiCredentialsConfig = result.json().await.map_err(|e| e.to_string())?;
    tracing::info!("received credentials: {:?}", credentials);
    Ok(CredentialsConfig {
        nc_admin_password: Some(nc_admin_password),
        primary_password: Some(primary_password),
        salt: Some(credentials.salt),
        // mfa_backup_codes: Some(credentials.mfa_backup_codes),
        disk_encryption_password: Some(credentials.disk_encryption_password),
        backup_encryption_password: Some(credentials.backup_password),
        backup_id : Some(rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(12).map(char::from)
            .collect()),
    })
}


#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum CredentialsConfigStep {
    Passwords,
    // SecondFactor,
    Backup,
    Verify,
    Summary
}

#[component]
pub fn CfgCredentials(error: Signal<Option<String>>,
                      on_back: EventHandler<MouseEvent>,
                      on_continue: EventHandler<MouseEvent>,
                      config: Signal<CredentialsConfig>,
                      status: Signal<ConfigStepStatus>) -> Element {

    let config_ref = Rc::new(RefCell::new(config));

    let nca_primary_password_proxy = use_signal(|| config().primary_password.unwrap_or(String::default()));
    use_effect(move || {
        let pw = match nca_primary_password_proxy().as_str() {
            "" => None,
            val => Some(val.to_string())
        };
        let mut newval = {
            config.peek().clone()
        };
        let peeked = config.peek().primary_password.clone();
        if peeked != pw {
            newval.primary_password = pw;
            config.set(newval);
        }
    });
    let nc_admin_password_proxy = use_signal(|| config().nc_admin_password.unwrap_or(String::default()));
    use_effect(move || {
        let pw = match nc_admin_password_proxy().as_str() {
            "" => None,
            val => Some(val.to_string())
        };
        let mut newval = {
            config.peek().clone()
        };
        if config.peek().nc_admin_password != pw {
            newval.nc_admin_password = pw;
            config.set(newval);
        }
    });

    let mut cred_config_step = use_signal(|| CredentialsConfigStep::Passwords);
    let mut is_backup_complete = use_signal(|| false);
    let mut are_credentials_confirmed = use_signal(|| false);
    use_effect(move || {
        {
            let old = status.peek();
            if old.completed && old.valid {
                is_backup_complete.set(true);
                are_credentials_confirmed.set(true);
            }
        }
        status.set({
            let old = status.peek();
            old.clone().with_visited(true)
        });
    });

    let primary_password_strength = use_memo(move || check_is_secure_password(nca_primary_password_proxy()));
    let nc_admin_password_strength = use_memo(move || check_is_secure_password(nc_admin_password_proxy()));

    let is_valid = use_resource(move || async move {
        match cred_config_step() {
            CredentialsConfigStep::Passwords => {
                primary_password_strength() == PasswordStrength::Strong
                    && nc_admin_password_strength() == PasswordStrength::Strong
            },
            // CredentialsConfigStep::SecondFactor => true,
            CredentialsConfigStep::Backup => is_backup_complete(),
            CredentialsConfigStep::Verify => are_credentials_confirmed(),
            CredentialsConfigStep::Summary => true
        }
    });

    let propagate_validation = use_effect(move || status.set({
        tracing::info!("config is '{:?}', updating status", is_valid());
        let is_valid = is_valid();
        let is_complete = is_valid.unwrap_or(false) && is_backup_complete() && are_credentials_confirmed();
        let old = status.peek();
        old.with_valid(is_valid.unwrap_or(false)).with_completed(is_complete)
    }));

    let advance = use_callback(move |evt: MouseEvent| async move {
        let next_step = match *cred_config_step.peek() {
            CredentialsConfigStep::Passwords => CredentialsConfigStep::Backup,
            CredentialsConfigStep::Backup => CredentialsConfigStep::Verify,
            CredentialsConfigStep::Verify => CredentialsConfigStep::Summary,
            CredentialsConfigStep::Summary => {
                on_continue.call(evt);
                return;
            }
        };
        if *cred_config_step.peek() == CredentialsConfigStep::Passwords {
            let config_clone = config.peek().clone();
            match (config_clone.primary_password, config_clone.nc_admin_password) {
                (Some(primary_password), Some(nc_admin_password)) => {
                    let result = derive_credentials_from_primary_password(primary_password, nc_admin_password).await;
                    match result {
                        Ok(newval) => {
                            config.set(newval);
                            cred_config_step.set(next_step);
                        },
                        Err(e) => {
                            error.set(Some(format!("Failed to save passwords: {}", e)));
                        }
                    }
                },
                _ => {
                    error.set(Some("primary password or nextcloud admin password is unset!".to_string()));
                }
            }
        } else {
            cred_config_step.set(next_step);
        }
    }
    );

    let success_icon = || {
        rsx!(Icon {
            class: "text-success",
            icon: hi_solid_icons::HiCheckCircle
        })
    };

    let failure_icon = || {
        rsx!(Icon {
            class: "text-error",
            icon: hi_solid_icons::HiXCircle
        })
    };

    let untouched_icon = || {
        rsx!(Icon {
            class: "text-default",
            icon: hi_solid_icons::HiMinusCircle
        })
    };

    rsx! {
        ul {
            class: "steps min-h-24",
            li {
                class: "step step-primary",
                "Primary Password"
            },
            li {
                class: "step",
                class: if cred_config_step() >= CredentialsConfigStep::Backup { "step-primary" },
                "Emergency Backup"
            },
            li {
                class: "step",
                class: if cred_config_step() >= CredentialsConfigStep::Verify { "step-primary" },
                "Confirm Credentials"
            },
            li {
                class: "step",
                class: if cred_config_step() >= CredentialsConfigStep::Summary { "step-primary" },
                "Summary"
            },

        },
        CfgConfigStep {
            back_button: rsx!(ConfigStepContinueButton{
                on_click:  move |evt| {
                    let next_step = match *cred_config_step.peek() {
                        CredentialsConfigStep::Passwords => {
                            on_back.call(evt);
                            return;
                        },
                        // CredentialsConfigStep::SecondFactor => CredentialsConfigStep::Passwords,
                        CredentialsConfigStep::Backup => CredentialsConfigStep::Passwords,
                        CredentialsConfigStep::Verify => CredentialsConfigStep::Backup,
                        CredentialsConfigStep::Summary => CredentialsConfigStep::Verify,
                    };
                    cred_config_step.set(next_step);
                },
                button_text: "Back"
            }),
            continue_button: rsx!(ConfigStepContinueButton{
                on_click: move |evt| {
                    advance(evt)
                },
                button_text: "Continue",
                disabled: !is_valid().unwrap_or(false)
            }),
            div {
                class: "flex-none p-2",
                if cred_config_step() == CredentialsConfigStep::Passwords {
                    div {
                        class: "pb-2",
                        // Accordion {
                        //     title: rsx!{
                        //         "Primary Password",
                        //         if primary_password_strength == PasswordStrength::Strong {
                        //             success_icon{}
                        //         } else {
                        //             failure_icon{}
                        //         }
                        //     },
                        //     class: "join-item",
                        //     name: "credential_step",
                        //     is_active: pw_accordion_active() == 0,
                        //     on_open: move |_| pw_accordion_active.set(0),
                        InputField {
                            r#type: InputType::Password(PasswordFieldConfig{
                                hide: false,
                                generator: true,
                                password_strength: Some(primary_password_strength())
                            }),
                            title: "Nextcloud Atomic Primary Password",
                            label: rsx!(div {
                                "This password will be used to log into the Nextcloud Atomic Admin interface.",
                            }),
                            value: nca_primary_password_proxy,
                            enable_copy_button: true,
                            prefix: rsx!(
                                Icon {
                                    class: "text-secondary h-1em opacity-50",
                                    icon: hi_solid_icons::HiKey,
                                    height: 30,
                                    width: 30
                                },)
                        },
                        // },
                        // Accordion {
                        //     title: rsx!{
                        //         "Nextcloud Admin Password",
                        //         if nc_admin_password_strength == PasswordStrength::Strong {
                        //             success_icon{}
                        //         } else {
                        //             failure_icon{}
                        //         }
                        //     },
                        //     class: "join-item",
                        //     name: "credential_step",
                        //     is_active: pw_accordion_active() == 1,
                        //     on_open: move |_| pw_accordion_active.set(1),
                        InputField {
                            class: "mt-4",
                            r#type: InputType::Password(PasswordFieldConfig{
                                hide: false,
                                generator: true, //pw_accordion_active() == 1,
                                password_strength: Some(nc_admin_password_strength())
                            }),
                            title: "Nextcloud Admin Password",
                            label: rsx!(div {
                                "This password will be used to log into Nextcloud as user ",
                                span {
                                    class: "italic",
                                    "admin"
                                },
                                "."
                            }),
                            value: nc_admin_password_proxy,
                            enable_copy_button: true,
                            prefix: rsx!(
                                Icon {
                                    class: "text-secondary h-1em opacity-50",
                                    icon: hi_solid_icons::HiKey,
                                    height: 30,
                                    width: 30
                                },)
                        },
                        // }
                    },
                } else if cred_config_step() == CredentialsConfigStep::Backup {
                    ConfigureCredentialsBackup {
                        credentials: config(),
                        is_backup_complete: is_backup_complete
                    }
                } else if cred_config_step() == CredentialsConfigStep::Verify {
                    ConfigureCredentialsConfirm {
                        credentials: config(),
                        is_confirmed: are_credentials_confirmed
                    }
                } else if cred_config_step() == CredentialsConfigStep::Summary {
                    CredentialsConfigSummary {
                        credentials: config()
                    }
                }

            }
        }
    }
}

#[component]
pub fn CredentialsConfigSummary(credentials: CredentialsConfig) -> Element {
    let mut salt = use_signal(|| "".to_string());
    // let mut backup_id = use_signal(|| "".to_string());
    let mut backup_encryption_password = use_signal(|| "".to_string());
    let mut disk_encryption_password = use_signal(|| "".to_string());
    use_effect(move || {
        salt.set(credentials.salt.clone().unwrap_or(String::default()));
        // backup_id.set(credentials.backup_id.clone().unwrap_or(String::default()));
        backup_encryption_password.set(credentials.backup_encryption_password.clone().unwrap_or(String::default()));
        disk_encryption_password.set(credentials.disk_encryption_password.clone().unwrap_or(String::default()));
    });

    rsx! {

        Alert {
            alert_color: Some(AlertColor::Warn),
            "The following credentials are automatically generated from your primary password. Please download or copy them and store them safely. You will need them to access your backups or your data in emergency cases."
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, password_strength: None}),
            disabled: true,
            title: "System Salt",
            label: rsx!(div {
                "This is an internal value used for deriving secret values from your primary password"
            }),
            value: salt,
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, password_strength: None}),
            disabled: true,
            title: "Disk Encryption Password",
            label: rsx!(div {
                "This password can be used to decrypt the disks of Nextcloud Atomic if automatic decryption fails"
            }),
            value: disk_encryption_password,
            enable_copy_button: true,
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{hide: false, generator: false, password_strength: None}),
            disabled: true,
            title: "Backup Encryption Password",
            label: rsx!(div {
                "This password is used to encrypt Nextcloud Atomic's incremental backups. You will need it to access your backups manually (or from a fresh Nextcloud Atomic Instance)"
            }),
            value: backup_encryption_password,
            enable_copy_button: true,
        },
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
                class: "border-base-100 w-full center shadow-sm p-2",
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