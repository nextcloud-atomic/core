use daisy_rsx::{Alert, AlertColor};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons;
use crate::components::configure_configstep::{CfgConfigStep, ConfigStepContinueButton};


#[derive(Props, Debug, PartialEq, Clone)]
struct Disk {
    label: String,
    size: u64,
    uuid: String
}

#[component]
pub fn CfgSetupStorage(error: Signal<Option<String>>, on_back: EventHandler<MouseEvent>, on_continue: EventHandler<MouseEvent>, on_validated: EventHandler<bool>) -> Element {
    let mut is_valid = use_signal(|| true);
    let selected_disk = use_signal(|| 0);
    let propagate_validation = use_effect(move || on_validated(is_valid()));

    let mock_disks: Vec<Disk> = vec![Disk {
        label: "Root Disk".to_string(),
        size: 10_000_000_000,
        uuid: "1234-1234-1234-1234-1234-1234".to_string()
    }, Disk {
        label: "My Disk 1".to_string(),
        size: 50_000_000_000,
        uuid: "1234-1234-1234-1234-1234-1234".to_string()
    }, Disk {
        label: "My Disk 2".to_string(),
        size: 100_000_000_000,
        uuid: "1234-1234-1234-1234-1234-1234".to_string()
    }];

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
                h2 {
                    class: "my-2",
                    "Choose a disk for store user data",
                },
                ul {
                    class: "list join join-vertical w-full cursor-pointer bg-base-100 rounded-box shadow-md",
                    for (i, disk) in mock_disks.iter().enumerate() {
                        DiskOption {
                            disk: disk.clone(),
                            disk_id: i,
                            selected_disk
                        }
                    }
                },
                if selected_disk() != 0 {

                    Alert {
                        class: "mt-4",
                        alert_color: Some(AlertColor::Warn),
                        {{ format!("Disk \"{}\" will be erased, encrypted and formatted.", mock_disks[selected_disk()].label) }}
                    },
                }

            }
        }
    }
}

#[component]
fn DiskOption(disk: Disk, disk_id: usize, selected_disk: Signal<usize>) -> Element {
    let is_selected = use_memo(move || disk_id == selected_disk());
    rsx! {
        li {
            class: "list-row flex flex-row py-2 join-item",
            class: if is_selected() { "bg-secondary text-secondary-content" },
            class: if !is_selected() { "bg-neutral text-neutral-content" },
            onclick: move |_| selected_disk.set(disk_id),
            div {
                class: "flex-none",
                Icon {
                    class: "text-secondary-content rounded-box mx-2",
                    icon: ld_icons::LdHardDrive,
                    height: 30,
                    width: 30,
                    title: "Disk Icon"
                }
            },
            div {
                class: "flex-1",
                div {
                    {{ disk.label }}
                },
                div {
                    class: "text-xs uppercase font-semibold opacity-60",
                    {{ format!("{} Bytes", disk.size) }}
                },
            }

        }
    }
}