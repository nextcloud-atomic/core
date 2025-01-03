#![allow(non_snake_case)]

use dioxus::prelude::*;
// use dioxus::fullstack::launch::LaunchBuilder;
use dioxus::fullstack::prelude::{server, ServerFnError};
use std::{env, io};
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use async_std::task::sleep;
use dioxus::fullstack::Config;
use log::{info, LevelFilter};
use serde::{Deserialize, Serialize};
use tera::Context;
use wasm_bindgen::JsValue;
use futures_util::StreamExt;
#[cfg(feature = "web")]
use {
    wasm_bindgen::prelude::wasm_bindgen,
    web_sys::js_sys::JsString,
    web_sys::{window, Window},
};

#[cfg(feature = "server")]
use {
    bollard::container::ListContainersOptions,
    bollard::models::ContainerSummary,
    bollard::Docker,
    caddy::CaddyClient,
    core::config::{NcAioConfig, NcaConfig},
    core::crypto::{Crypto, CryptoValueProvider},
    hyper::Method,
    sd_notify::notify,
    sd_notify::NotifyState,
    std::process::exit,
    std::time::Duration
};
#[cfg(not(feature = "server"))]
use instant::Duration;
#[cfg(feature = "server")]
use core::templating::render_template;

#[cfg(feature = "web")]
fn set_server_address(launcher: LaunchBuilder<()>) -> LaunchBuilder<()> {
    launcher
}

#[cfg(feature = "server")]
fn main() {
    // dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    // let mut launcher = LaunchBuilder::new(app);
    // launcher = set_server_address(launcher);
    // launcher.launch();
    // let config_path = PathBuf::from(env::var("NCA_CONFIG_TARGET")
    //                                     .expect("NCA_CONFIG_TARGET must be set"));

    // tokio::runtime::Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap()
    //     .block_on(async {
    //         if config_path.join("ncatomic.json").exists() {
    //             caddy_enable_nextcloud().await.expect("Failed to enable nextcloud page");
    //             notify_service_successful().expect("Failed to notify systemd service completion");
    //             terminate().await.expect("Failed to terminate service");
    //         } else {
    //             caddy_enable_activation_page().await.expect("Failed to configure caddy");
    //         }
    //     });
    let cfg = Config::default()
        .addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 
            8080));
    LaunchBuilder::new()
        .with_cfg(cfg)
        .launch(app)
    // tokio::signal::unix::signal(signal::unix::SignalKind::terminate()).expect("Failed to init signal handler").recv().await
}

#[cfg(not(feature = "server"))]
fn main() {
    launch(app)
}

fn print_err<E: ToString>(e: E) -> ServerFnError {
    eprintln!("{}", e.to_string());
    ServerFnError::new(e)
}

#[cfg(feature = "server")]
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
async fn activate_ncatomic(user_pass: String) -> Result<(), ServerFnError> {
    let crypto = Crypto::new(ncatomic_core::NCATOMIC_VERSION, &user_pass).map_err(ServerFnError::new)?;
    let config = NcaConfig::new(ncatomic_core::NCATOMIC_VERSION, &crypto).map_err(ServerFnError::new)?;

    let config_template_base_path = PathBuf::from(env::var("NCA_CONFIG_SOURCE")
        .map_err(print_err)?);
    let config_render_base_path = PathBuf::from(env::var("NCA_CONFIG_TARGET")
        .map_err(print_err)?);
    config.save(config_render_base_path.join("ncatomic.json")).map_err(ServerFnError::new)?;
    render_aio_config(config.nc_aio,
                      &crypto,
                      config_template_base_path.join("nextcloud-aio"),
                      config_render_base_path.join("nextcloud-aio"))?;
    notify_service_successful().
        map_err(|e| ServerFnError::new("Could not complete activation: ".to_string() + &e.to_string()))?;
    Ok(())
    //    .expect("Failed to create master key from password");
}

#[cfg(feature = "server")]
fn notify_service_successful() -> io::Result<()> {
    notify(true, &[NotifyState::Ready])
}

#[server]
async fn terminate() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
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
        if !names.iter().any(|name| name.starts_with("/nextcloud-aio") || name == "nca-caddy") {
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
    let required_container_names = [
        String::from("nextcloud-aio-apache"),
        String::from("nextcloud-aio-nextcloud")
    ];
    let is_ready = required_container_names.iter()
        .all(|name| containers.iter()
            .any(|container| {
                println!("{} - {}",
                         container.names.clone().unwrap_or_default().first()
                             .unwrap_or(&String::from("unknown")),
                         container.state.clone().unwrap());
                container.state.clone().unwrap_or("unknown".into()) == "running" &&
                    container.names.clone().unwrap_or_default().iter()
                        .any(|s| s.contains(name))
            })
    );
    println!("isReady? {is_ready}");
    Ok(ContainerStatusResult {
        ready: is_ready,
        docker_version: version.version.unwrap(),
        containers: container_strings,
    })
}

#[cfg(feature = "server")]
async fn caddy_enable_activation_page() -> anyhow::Result<()> {
    let caddy_cli = CaddyClient::new(&env::var("CADDY_ADMIN_SOCKET")?)?;
    let mut f = File::options().read(true).open("/resource/caddy/default_ncatomic_activation.json")?;
    let mut cfg = String::new();
    f.read_to_string(&mut cfg)?;
    caddy_cli.set_caddy_servers(cfg).await?;
    Ok(())
}

#[server]
async fn caddy_enable_nextcloud() -> Result<(), ServerFnError>{
    let caddy_cli = CaddyClient::new(&env::var("CADDY_ADMIN_SOCKET")?)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let mut f = File::options().read(true).open("/resource/caddy/default_nc_aio.json")?;
    let mut cfg = String::new();
    f.read_to_string(&mut cfg)?;
    caddy_cli.set_caddy_servers(cfg).await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[cfg(feature = "web")]
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
                        println!("container status: {containers_status:?}");
                        if ready {
                            match caddy_enable_nextcloud().await {
                                Ok(_) => {
                                    if let Err(e) = terminate().await {
                                        error_message.set(Some("Failed to stop server: ".to_string() + &e.to_string()))
                                    }
                                },
                                Err(e) => {
                                    error_message.set(Some("Failed to configure reverse proxy: ".to_string() + &e.to_string()))
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some("Failed to start Nextcloud services: ".to_string() + &e.to_string()));
                    }
                        
                }
            }
        }
    });
    match nextcloud_reachable.take() {
        false => rsx! {
            div {
                "Set the Nextcloud Atomic admin password:",
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
                        if let Err(e) = activate_ncatomic(userpass.take()).await {
                            status.set(e.to_string());
                        } else {
                            nc_status_check.send(true);
                            status.set("Nextcloud Atomic activated successfully! - Waiting for services to start".to_string());
                            activated.set(true)
                        }
                    }

                },
                "Activate Nextcloud Atomic",
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


#[cfg(feature = "server")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use tera::{Context, Tera};
    use core::config::NcAioConfig;
    use core::crypto::{Crypto, CryptoValueProvider};

    #[test]
    fn render_aio_templates() {
        let aio_cfg = NcAioConfig::default();
        let crypto = Crypto::new(ncatomic_core::NCATOMIC_VERSION, "testpw")
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
