use dioxus::prelude::*;
use dioxus::prelude::component;
use model::disk::DiskInfo;
use crate::layout::Layout;

#[derive(Props, Clone, PartialEq)]
pub struct IndexPageProps {
    pub disks: Vec<DiskInfo>,
}
//
// #[component]
// pub fn Disks(props: &DisksProps) -> Element {
//     rsx! {}
// }

#[component]
pub fn IndexPage(props: IndexPageProps) -> Element {
    rsx! {
        Layout {
            title: "Disks",
            table {
                thead {
                    tr {
                        th { "Device" }
                        th { "Mountpoint" }
                        th { "Kind" }
                        th { "size" }
                        th { "available space" }
                        th { "readonly?" }
                        th { "removable?" }
                    }
                }
                tbody {
                    for disk in props.disks {
                        tr {
                            td { strong { "{disk.name}" } }
                            td { "{disk.mount_point.unwrap_or(String::new())}" }
                            td { "{disk.kind}" }
                            td { "{disk.total_space}" }
                            td { "{disk.available_space}" }
                            td { "{disk.is_read_only}" }
                            td { "{disk.is_removable}" }
                        }
                    }
                }
            }
        }
    }
}
