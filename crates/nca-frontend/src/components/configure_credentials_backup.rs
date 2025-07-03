use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons;
use dioxus_logger::tracing;
use web_sys::{js_sys, wasm_bindgen, window, Url};
use crate::components::accordion::Accordion;
use crate::components::form::{InputField, InputType, PasswordFieldConfig};
use crate::{base_url, do_post, ConfigStepStatus};
use crate::configure_credentials::CredentialsConfig;

#[component]
pub fn ConfigureCredentialsBackup(credentials: CredentialsConfig,
                                  mut is_backup_complete: Signal<bool>) -> Element {

    tracing::info!("credentials: {:?}", credentials.clone());

    let creds_1 = credentials.clone();
    let mut backup_html: Signal<Option<String>> = use_signal(|| None);
    let js_connect = use_resource( move || {
        let creds = creds_1.clone();
        async move {
            let mut eval = document::eval(include_str!("../../resource/nca-credentials-backup-encrypt.js"));
            eval.send(creds).expect("Failed to send data to javascript");
            let credentials: String = eval.recv().await.unwrap();
            tracing::info!("got credentials: \n{}\n=====", credentials);
            let html = include_str!("../../resource/nca-credentials-backup.html").replace(
                "<!-- GENERATED PARAMETERS PLACEHOLDER -->",
                format!("<script type=\"application/javascript\">\n{credentials}\n</script>").as_str(),
            );
            backup_html.set(Some(html));
        }
    });

    let backup_html_download_url: Memo<Option<String>> = use_memo(move || match backup_html() {
        None => None,
        Some(html) => {
            let js_val = js_sys::Array::from_iter(std::iter::once(js_sys::JsString::from(html)));
            let blob = web_sys::Blob::new_with_str_sequence(&**js_val).unwrap();
            Some(Url::create_object_url_with_blob(&blob).unwrap())
        }
    });

    let creds_2 = credentials.clone();
    let backup_plain_download_url: Memo<String> = use_memo(move || {
        let js_val = js_sys::Array::from_iter(std::iter::once(js_sys::JsString::from(
            serde_json::to_string(&creds_2).unwrap()
        )));
        let blob = web_sys::Blob::new_with_str_sequence(&**js_val).unwrap();
        Url::create_object_url_with_blob(&blob).unwrap()
    });

    let mut selected_backup_option = use_signal(|| 0);

    rsx!{
        if let Some(url) = backup_html_download_url() {
            div {
                class: "pb-2 join join-vertical",
                div {
                    "Choose a credentials backup format to download."
                },
                Accordion {
                    title: rsx!{
                        div {
                            class: "flex flex-col",
                            h2 { "Encrypted Credentials Backup" },
                        }
                    },
                    class: "join-item",
                    name: "credentials_backup_option",
                    is_active: selected_backup_option() == 0,
                    on_open: move |_| selected_backup_option.set(0),
                    div {
                        class: "text-center",
                        div {
                            class: "alert alert-warning mb-4",
                            span {
                                class: "text-xs",
                                "The credentials are encrypted and cannot be accessed without the ",
                                "primary password. If you loose your password, however, you will loose ",
                                "access to the backup.",
                                "Open the downloaded html file in any browser and enter your primary ",
                                "password to unlock the backup."
                            }
                        },
                        a {
                            class: "btn btn-secondary mx-auto",
                            href: "{url}",
                            download: "nactomic-credentials-backup.html",
                            onclick: move |_| is_backup_complete.set(true),
                            "Download Encrypted Credentials Backup"
                        }
                    }
                }
                Accordion {
                    title: rsx!{
                        div {
                            class: "flex flex-col",
                            h2 { "Plaintext Credentials Backup" },
                        }
                    },
                    class: "join-item",
                    name: "credentials_backup_option",
                    is_active: selected_backup_option() == 1,
                    on_open: move |_| selected_backup_option.set(1),
                    div {
                        class: "text-center",
                        div {
                            class: "alert alert-warning mb-4",
                            span {
                                class: "text-xs",
                                "You are responsible for securely storing and protecting the ",
                                "credentials backup (e.g. in a password manager)."
                            }
                        },
                        a {
                            class: "btn btn-secondary mx-auto",
                            href: backup_plain_download_url,
                            download: "nactomic-credentials-backup.txt",
                            onclick: move |_| is_backup_complete.set(true),
                            "Download Unencrypted Credentials Backup"
                        }
                    }
                }
            }

        } else {
            div {
                "Creation Credentials Backup ..."
            }
        }
    }
}

#[component]
pub(crate) fn ConfigureCredentialsConfirm(credentials: CredentialsConfig,
                                          is_confirmed: Signal<bool>) -> Element {
    let mut backup_id = use_signal(|| "".to_string());
    let mut primary_pw = use_signal(|| "".to_string());

    let credentials_cpy = credentials.clone();
    use_effect(move || if *is_confirmed.peek() {
        backup_id.set(credentials_cpy.backup_id.clone().unwrap_or(String::default()));
        primary_pw.set(credentials_cpy.primary_password.clone().unwrap_or(String::default()));
    });

    let check_is_valid = use_effect(move || {
        is_confirmed.set(Some(backup_id()) == credentials.backup_id && Some(primary_pw()) == credentials.primary_password)
    });

    rsx! {
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{
                hide: true,
                generator: false,
                password_strength: None
            }),
            title: "Backup ID",
            label: rsx!(div {
                "Open your backup, find the 'backup ID' and enter it here to confirm that your backup is working:"
            }),
            value: backup_id,
            enable_copy_button: false,
            prefix: rsx!(
                Icon {
                    class: "text-secondary h-1em opacity-50",
                    icon: hi_solid_icons::HiIdentification,
                    height: 30,
                    width: 30
                }
            )
        },
        InputField {
            r#type: InputType::Password(PasswordFieldConfig{
                hide: true,
                generator: false,
                password_strength: None
            }),
            title: "Primary Password",
            label: rsx!(div {
                "Enter your Nextcloud Atomic Primary Password"
            }),
            value: primary_pw,
            enable_copy_button: false,
            prefix: rsx!(
                Icon {
                    class: "text-secondary h-1em opacity-50",
                    icon: hi_solid_icons::HiKey,
                    height: 30,
                    width: 30
                }
            )
        }
    }
}
