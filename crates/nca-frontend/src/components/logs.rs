use std::collections::HashMap;
use std::thread::Scope;
use dioxus::document;
use dioxus::hooks::{use_coroutine, use_signal, UnboundedReceiver};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
struct LogMessage {
    fields: HashMap<String, String>,
    namespace: Option<String>,
    message: String
}
impl LogMessage {
    fn systemd_unit(&self) -> String {
        let service_name_fallback = "unknown".to_string();
        let service_full_name = self.fields.get("_SYSTEMD_UNIT")
            .unwrap_or(&service_name_fallback);
        match service_full_name
            .strip_suffix(".service") {
            Some(s) => s.to_string(),
            None => service_full_name.to_string()
        }
    }

    fn render_inner_html(&self) -> String {
        if let Some(container) = self.fields.get("_CONTAINER_NAME") {
            format!("[{}]::<b>{}</b>:: {}", self.systemd_unit(), container, self.message)
        } else {
            format!("[{}]:: {}", self.systemd_unit(), self.message)
        }.to_string()
    }
}
#[derive(Deserialize, Debug, Clone)]
struct JsMessage {
    error: Option<String>,
    message: Option<LogMessage>
}

#[component]
pub fn Logs() -> Element {

    let mut logs: Signal<Vec<LogMessage>> = use_signal(|| vec![]);
    let js_connect = use_coroutine(move |rx: UnboundedReceiver<bool>| async move {
        let mut eval = document::eval(include_str!("../../dist/_grpc_client.js"));
        loop {
            let msg: String = eval.recv().await.unwrap();
            #[cfg(debug_assertions)]
            tracing::info!("Message received: {}", msg);
            if let Ok(data) = serde_json::from_str::<JsMessage>(&*msg) {
                match data.error {
                    Some(error) => {
                        tracing::error!("Received error from grpc: {error}")
                    }
                    None => {
                        let message = data.message
                            .expect("Err: Neither message nor error found in js message");
                        logs.push(message);
                    }
                }
            } else {
                tracing::error!("Failed to deserialize message: {msg}")
            };
        };
    });

    rsx! {
        div {
            class: "logstream mockup-code box-border flex-1 min-h-20vh overflow-y-scroll m-2",
            for (i, log) in logs.iter().enumerate() {
                pre {
                    "data-prefix": "{i+1}",
                    "[{log.systemd_unit()}]=>",
                    if let Some(container) = log.fields.get("_CONTAINER_NAME") {
                        b {
                            "{container}::"
                        }
                    },
                    " {log.message}"
                }
            }
        }
    }
}