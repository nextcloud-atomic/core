#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_fullstack::launch::LaunchBuilder;
use dioxus_fullstack::prelude::{server, ServerFnError};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::rc::Rc;
use async_std::prelude::StreamExt;
use bollard::Docker;
use bollard::models::ContainerSummary;
use dioxus::html::summary;
use serde::{Serialize, Serializer};
use tera::Context;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{window, Window};
use web_sys::js_sys::JsString;
use ncp_core::config::{NcAioConfig, NcpConfig};
use ncp_core::crypto::{Crypto, CryptoValueProvider};
use ncp_core::error::NcpError;

use ncp_core::templating::render_template;

#[cfg(feature = "ssr")]
use {std::time::Duration, std::process::exit};
use regex::Regex;

#[cfg(feature = "ssr")]
fn set_server_address(launcher: LaunchBuilder<()>) -> LaunchBuilder<()> {
    launcher.addr(SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 8080))
}

#[cfg(not(feature = "ssr"))]
fn set_server_address(launcher: LaunchBuilder<()>) -> LaunchBuilder<()> {
    launcher
}

fn main() {
    let mut launcher = LaunchBuilder::new(app);
    launcher = set_server_address(launcher);
    launcher.launch();
    // tokio::signal::unix::signal(signal::unix::SignalKind::terminate()).expect("Failed to init signal handler").recv().await
}

fn print_err<E: std::error::Error>(e: E) -> E {
    eprintln!("{:?}", e);
    e
}

fn render_aio_config(cfg: NcAioConfig, crypto: &Crypto, aio_template_path: PathBuf, aio_render_path: PathBuf) -> Result<(), ServerFnError> {
    let mut tera_ctx = Context::new();
    tera_ctx.insert("NC_AIO_CONFIG", &cfg);
    tera_ctx.insert("NC_AIO_SECRETS", &cfg.get_crypto_value(crypto)?);
    render_template(tera_ctx.clone(),
                    aio_template_path.join("defaults.env.j2"),
                    aio_render_path.join(".env"))
        .map_err(print_err)?;
    render_template(tera_ctx,
                    aio_template_path.join("compose.yaml.j2"),
                    aio_render_path.join("compose.yaml"))
        .map_err(print_err)?;
    Ok(())
}

#[server]
async fn activate_ncp(user_pass: String) -> Result<(), ServerFnError> {
    let crypto = Crypto::new(ncp_core::NCP_VERSION, &user_pass)?;
    let config = NcpConfig::new(ncp_core::NCP_VERSION, &crypto)?;

    let config_template_base_path = PathBuf::from(env::var("NCP_CONFIG_SOURCE")
        .map_err(print_err)?);
    let config_render_base_path = PathBuf::from(env::var("NCP_CONFIG_TARGET")
        .map_err(print_err)?);
    config.save(config_render_base_path.join("ncp.json.j2"))?;
    render_aio_config(config.nc_aio,
                      &crypto,
                      config_template_base_path.join("nextcloud-aio"),
                      config_render_base_path.join("nextcloud-aio"))?;
    Ok(())
    //    .expect("Failed to create master key from password");
}

#[server]
async fn terminate() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            exit(0);
        });
    }
    Ok(())
}

struct ContainerStatusResult {
    containers: Vec<String>,
    ready: bool,
    docker_version: String,
}

impl Display for ContainerStatusResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = (match self.ready {
            true => "Waiting for containers:\n=> ",
            false => "All containers started!\n=> "
        }).to_string()
            + self.containers.join("\n=> ").as_str()
            + format!("\n\n (docker version: {})", self.docker_version).as_str();
        write!(f, "{}", str)
    }
}

#[server]
async fn check_aio_started() -> Result<ContainerStatusResult, ServerFnError> {
    let docker = Docker::connect_with_socket_defaults()?;
    let version = docker.version().await?;
    let containers = docker.list_containers().await?
        .iter().filter_map(|summary| {
        format!("{}: {} ({}s)", summary.image, summary.status, summary.created)
    });
    ContainerStatusResult {
        ready: false,
        docker_version: version.version.unwrap(),
        containers,
    }
}

#[wasm_bindgen]
pub fn get_location() -> Result<JsString, JsValue> {
    let window = web_sys::window().unwrap();
    let loc = window.location();
    Ok(loc.to_string())
    //Ok((loc.protocol()?, loc.host()?, loc.port()?, loc.pathname()?))
}

pub fn app(cx: Scope) -> Element {
    let mut userpass = use_state(cx, || "".to_string());
    let mut status = use_state(cx, || "".to_string());
    let mut error_message: &UseState<Option<String>> = use_state(cx, || None);
    let mut activated = use_state(cx, || false);
    let mut nextcloud_reachable = use_state(cx, || false);
    let mut containers_status: &UseState<Option<ContainerStatusResult>> = use_state(cx, || None);
    // let eval = use_eval(cx);
    // let nc_status_check = use_coroutine(cx, |mut rx: UnboundedReceiver<bool>|  {
    //     to_owned![nextcloud_reachable];
    //     async move {
    //         rx.next().await;
    //         let loc = get_location().expect("Failed to get window.location");
    //         let re = Regex::new(r"/$");
    //         let loc_str = re.unwrap().replace(&loc.as_string().unwrap(), "").to_string();
    //         async_std::task::sleep(instant::Duration::from_millis(5000)).await;
    //         loop {
    //             async_std::task::sleep(instant::Duration::from_millis(1000)).await;
    //             let response = reqwest::get(format!("{}/login", loc_str)).await;
    //
    //             if response.is_ok() {
    //                 let result = response.unwrap();
    //                 if result.status() == 200 && result.text().await.unwrap().contains("_oc_debug") {
    //                     nextcloud_reachable.set(true);
    //                 }
    //             }
    //         }
    //     }
    // });
    let nc_status_check = use_coroutine(cx, |mut rx: UnboundedReceiver<ContainerStatusResult>| {
        to_owned![containers_status, error_message];
        async move {
            rx.next().await;
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                match check_aio_started().await {
                    Ok(result) => {
                        containers_status.set(Some(result))
                    }
                    Err(e) => {
                        error_message.set(Some(e.to_string()))
                    }
                }
            }
        }
    });
    cx.render(match nextcloud_reachable.get() {
        false => rsx! {
            div {
                "Set the NCP master password:",
            },
            input {
                name: "userpass",
                value: "{userpass}",
                oninput: move |evt| userpass.set(evt.value.clone()),
            },
            button {
                r#type: "button",
                onclick: move |evt| {
                    to_owned![status, userpass, activated, nc_status_check];
                    async move {
                        if let Err(e) = activate_ncp(userpass.current().to_string()).await {
                            status.set(e.to_string());
                        } else {
                            nc_status_check.send(true);
                            //terminate().await.expect("Failed to stop server");
                            status.set("NCP activated successfully! - Waiting for services to start".to_string());
                            activated.set(true)
                        }
                    }

                },
                "Activate NCP",
            },
            div {
                "{status}",
            },
            pre {
                match containers_status {
                    Some(val) => val.to_string(),
                    None => ""
                }
            }

        },
        true => rsx! {
            div {
                "Nextcloud has started."
            },
            a {
                href: "http://localhost:1080/login",
                "Open Nextcloud"
            }
        }
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use tera::{Context, Tera};
    use ncp_core::config::NcAioConfig;
    use ncp_core::crypto::{Crypto, CryptoValueProvider};

    #[test]
    fn render_aio_templates() {
        let aio_cfg = NcAioConfig::default();
        let crypto = Crypto::new(ncp_core::NCP_VERSION, "testpw")
            .expect("failed to create crypto");
        let mut tera_ctx = Context::new();
        tera_ctx.insert("NC_AIO_CONFIG", &aio_cfg);
        tera_ctx.insert("NC_AIO_SECRETS", &aio_cfg.get_crypto_value(&crypto)
            .expect("Failed to retrieve secrets"));

        let mut f = File::open(PathBuf::from("../../resource/templates/nextcloud-aio/defaults.env.j2"))
            .expect("failed to open defaults.env.j2");
        let mut templ = String::new();
        f.read_to_string(&mut templ).expect("failed to read defaults.env.j2");
        let result = Tera::one_off(&templ, &tera_ctx, false)
            .expect("failed to render defaults.env.j2");
        println!("{result}");

        let mut f = File::open(PathBuf::from("../../resource/templates/nextcloud-aio/compose.yaml.j2"))
            .expect("failed to open compose.yaml.j2");
        let mut templ = String::new();
        f.read_to_string(&mut templ).expect("failed to read compose.yaml.j2");
        let rendered = Tera::one_off(&templ, &tera_ctx, false)
            .expect("failed to render compose.yaml.j2");
        println!("{result}");
    }
}
