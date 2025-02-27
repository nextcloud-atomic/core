#![allow(non_snake_case)]

use std::time::Duration;
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::hi_solid_icons};
use dioxus_logger::tracing;
use nca_system_api::systemd::types::ServiceStatus;
use crate::base_url;
use crate::components::nc_startup::NcStartup;

#[derive(Props, Clone, PartialEq)]
pub struct ServiceStatusProps {
    service_name: String,
    on_activating: Element,
    on_active: Element,
    on_failed: Element,
    on_inactive: Option<Element>,
    on_deactivating: Option<Element>,
    on_api_error: Option<Element>,
    error_action: Option<Element>,
    success_action: Option<Element>,
}

pub fn ServiceStatus(props: ServiceStatusProps) -> Element {

    let mut service_status: Signal<Option<nca_system_api::systemd::types::ServiceStatus>> = use_signal(|| None);
    let mut service_name: Signal<String> = use_signal(|| props.service_name.clone());
    let nc_aio_status_future = use_coroutine(move |rx: UnboundedReceiver<bool>| async move {
        to_owned![service_status];
        let request_url = format!("{}/api/setup/service/{}.service", base_url(), service_name.peek());
        loop {
            tracing::info!("requesting {}", request_url);
            let status = match reqwest::get(&request_url).await {
                Err(e) => {
                    tracing::error!("Failed to retrieve service status: {:?}", e);
                    None
                },
                Ok(response) => match response.json::<nca_system_api::systemd::types::ServiceStatus>().await {
                    Err(e) => {
                        tracing::error!("Failed to parse service status response: {:?}", e);
                        None
                    },
                    Ok(status) => Some(status)
                }
            };
            service_status.set(status);
            async_std::task::sleep(Duration::from_secs(5)).await;
        };
    });
    
    let fallback_error_elem = rsx! {
        h2 {
            class: "card-title",
            Icon {
                class: "text-error",
                icon: hi_solid_icons::HiExclamationCircle,
                height: 30,
                width: 30
            },
            "Internal Server Error while fetching status of {props.service_name}" 
        }
    };
    
    let fallback_inactive_elem = rsx! {
        h2 {
            class: "card-title",
            "{props.service_name} is inactive"
        }
    };
    
    rsx! {
        div {
            class: "card card-border bg-base-100 w-80% shadow-sm flex-0",
            div {
                class: "card-body",
                match service_status() {
                    Some(ServiceStatus::ACTIVE) => {props.on_active},
                    Some(ServiceStatus::ACTIVATING) => {props.on_activating},
                    Some(ServiceStatus::FAILED) => {props.on_failed},
                    Some(ServiceStatus::INACTIVE) => {props.on_inactive.unwrap_or(fallback_inactive_elem)},
                    Some(ServiceStatus::DEACTIVATING) => {props.on_deactivating.unwrap_or(props.on_failed)},
                    None => {props.on_api_error.unwrap_or(fallback_error_elem)}
                }
                match service_status() {
                    Some(ServiceStatus::FAILED) | Some(ServiceStatus::DEACTIVATING) => {
                        match props.error_action {
                            Some(action) => rsx!{
                                div {
                                    class: "justify-end card-action",
                                    {action}
                                }
                            },
                            None => rsx!()
                        }
                    },
                    Some(ServiceStatus::ACTIVE) => {
                        match props.success_action {
                            Some(action) => rsx!{
                                div {
                                    class: "justify-end card-action",
                                    {action}
                                }
                            },
                            None => rsx!()
                        }
                    },
                    _ => {rsx!()}
                }
            },
        }
    }
}