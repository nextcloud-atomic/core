#![allow(non_snake_case)]

use dioxus::prelude::*;
// use dioxus::fullstack::launch::LaunchBuilder;
use dioxus::fullstack::prelude::{server, ServerFnError};
use dioxus::prelude::*;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::rc::Rc;
use async_std::task::sleep;
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize, Serializer};
use tera::Context;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{window, Window};
use web_sys::js_sys::JsString;
#[cfg(feature = "ssr")]
use core::config::{NcAioConfig, NcpConfig};
#[cfg(feature = "ssr")]
use core::crypto::{Crypto, CryptoValueProvider};
use regex::Regex;
use futures_util::StreamExt;

#[cfg(feature = "ssr")]
use {
    std::time::Duration,
    std::process::exit,
    ncp_core::templating::render_template,
    sd_notify::notify,
    sd_notify::NotifyState,
    bollard::Docker,
    bollard::models::ContainerSummary,
    bollard::container::ListContainersOptions
};
#[cfg(not(feature = "ssr"))]
use {
   instant::Duration
};


// #[cfg(feature = "ssr")]
// fn set_server_address(launcher: LaunchBuilder<()>) -> LaunchBuilder<()> {
//     launcher.addr(SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 8080))
// }

#[cfg(not(feature = "ssr"))]
fn set_server_address(launcher: LaunchBuilder<()>) -> LaunchBuilder<()> {
    launcher
}

fn main() {
    // dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    // let mut launcher = LaunchBuilder::new(app);
    // launcher = set_server_address(launcher);
    // launcher.launch();
    launch(app);
    // tokio::signal::unix::signal(signal::unix::SignalKind::terminate()).expect("Failed to init signal handler").recv().await
}

fn print_err<E: ToString>(e: E) -> ServerFnError {
    eprintln!("{}", e.to_string());
    ServerFnError::new(e)
}

#[cfg(feature = "ssr")]
fn render_aio_config(cfg: NcAioConfig, crypto: &Crypto, aio_template_path: PathBuf, aio_render_path: PathBuf) -> Result<(), ServerFnError> {
    let mut tera_ctx = Context::new();
    tera_ctx.insert("NC_AIO_CONFIG", &cfg);
    tera_ctx.insert("NC_AIO_SECRETS", &cfg.get_crypto_value(crypto).map_err(|e| ServerFnError::new(e.to_string()))?);
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
let crypto = Crypto::new(ncp_core::NCP_VERSION, &user_pass).map_err(ServerFnError::new)?;
    let config = NcpConfig::new(ncp_core::NCP_VERSION, &crypto).map_err(ServerFnError::new)?;

    let config_template_base_path = PathBuf::from(env::var("NCP_CONFIG_SOURCE")
        .map_err(print_err)?);
    let config_render_base_path = PathBuf::from(env::var("NCP_CONFIG_TARGET")
        .map_err(print_err)?);
    config.save(config_render_base_path.join("ncp.json.j2")).map_err(ServerFnError::new)?;
    render_aio_config(config.nc_aio,
                      &crypto,
                      config_template_base_path.join("nextcloud-aio"),
                      config_render_base_path.join("nextcloud-aio"))?;
    notify(true, &[NotifyState::Ready]);
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContainerStatusResult {
    containers: Vec<String>,
    ready: bool,
    docker_version: String,
}

impl Display for ContainerStatusResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = (match self.ready {
            false => "Waiting for containers:\n=> ",
            true => "All containers started!\n=> "
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
    let options = Some(ListContainersOptions::<String>{
        all: true,
        ..Default::default()
    });
    let containers = docker.list_containers(options).await?;
    let container_strings = containers
        .iter().filter_map(move |s| {
        let names = s.names.clone().unwrap_or(vec!());
        if !names.iter().any(|name| name.starts_with("/nextcloud-aio") || name == "ncp-caddy") {
            println!("names: [{}]", names.join(", "));
            return None
        }
        Some(format!("{}/{}: {} ({:?}s) - [{}]",
                s.image_id.clone().unwrap_or("unknown".into()),
                s.image.clone().unwrap_or("unknown".into()),
                s.status.clone().unwrap_or("unknown".into()),
                s.created.unwrap_or(0).to_string(),
                s.names.clone().unwrap_or(vec!()).join(", ")
        ))
    }).collect();
    let is_ready = containers.iter().any(|container| {
        println!("{}", container.state.clone().unwrap());
        container.state.clone().unwrap_or("unknown".into()) == "running" &&
            container.names.clone().unwrap_or(vec![]).iter().any(|s| s == "/nextcloud-aio-apache")
    });
    Ok(ContainerStatusResult {
        ready: is_ready,
        docker_version: version.version.unwrap(),
        containers: container_strings,
    })
}

#[server]
async fn caddy_enable_nextcloud() -> Result<(), ServerFnError>{
    Ok(())
}

#[wasm_bindgen]
pub fn get_location() -> Result<JsString, JsValue> {
    let window = web_sys::window().unwrap();
    let loc = window.location();
    Ok(loc.to_string())
    //Ok((loc.protocol()?, loc.host()?, loc.port()?, loc.pathname()?))
}

pub fn app() -> Element {
    let mut userpass = use_signal(|| "".to_string());
    let mut status = use_signal(|| "".to_string());
    let mut error_message: Signal<Option<String>> = use_signal(|| None);
    let mut activated = use_signal(|| false);
    let mut nextcloud_reachable = use_signal(|| false);
    let mut containers_status: Signal<Option<ContainerStatusResult>> = use_signal(|| None);
    let nc_status_check = use_coroutine(|mut rx: UnboundedReceiver<bool>| {
        to_owned![containers_status, error_message];
        async move {
            rx.next().await;
            let mut ready = false;
            while !ready {
                sleep(Duration::from_millis(1000)).await;
                match check_aio_started().await {
                    Ok(result) => {
                        ready = result.ready;
                        containers_status.set(Some(result));
                    }
                    Err(e) => {
                        error_message.set(Some(e.to_string()));
                    }
                }
            }
        }
    });
    match nextcloud_reachable.take() {
        false => rsx! {
            div {
                "Set the NCP master password:",
            },
            input {
                name: "userpass",
                value: "{userpass}",
                oninput: move |evt| userpass.set(evt.value()),
            },
            button {
                r#type: "button",
                onclick: move |evt| {
                    to_owned![status, userpass, activated, nc_status_check];
                    async move {
                        if let Err(e) = activate_ncp(userpass.take()).await {
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
                match containers_status.take() {
                    Some(val) => val.to_string(),
                    None => "".to_string()
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
    }
}


#[cfg(feature = "ssr")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use tera::{Context, Tera};
    use core::config::NcAioConfig;
    use core::crypto::{Crypto, CryptoValueProvider};

    #[test]
    fn render_aio_templates() {
        let aio_cfg = NcAioConfig::default();
        let crypto = Crypto::new(ncp_core::NCP_VERSION, "testpw")
            .expect("failed to create crypto");
        let mut tera_ctx = Context::new();
        tera_ctx.insert("NC_AIO_CONFIG", &aio_cfg);
        tera_ctx.insert("NC_AIO_SECRETS", &aio_cfg.get_crypto_value(&crypto)
            .expect("Failed to retrieve secrets"));

        let mut f = File::open(env::current_dir().unwrap().join("resource/templates/nextcloud-aio/defaults.env.j2"))
            .expect("failed to open defaults.env.j2");
        let mut templ = String::new();
        f.read_to_string(&mut templ).expect("failed to read defaults.env.j2");
        let result = Tera::one_off(&templ, &tera_ctx, false)
            .expect("failed to render defaults.env.j2");
        println!("{result}");

        let mut f = File::open(env::current_dir().unwrap().join("resource/templates/nextcloud-aio/compose.yaml.j2"))
            .expect("failed to open compose.yaml.j2");
        let mut templ = String::new();
        f.read_to_string(&mut templ).expect("failed to read compose.yaml.j2");
        let rendered = Tera::one_off(&templ, &tera_ctx, false)
            .expect("failed to render compose.yaml.j2");
        println!("{result}");
    }
}
