#![allow(non_snake_case)]

use std::{fmt, fs};
use std::fmt::{Formatter};
use std::fs::File;
use std::num::NonZeroU32;
use std::path::PathBuf;
use ring::{digest, pbkdf2, hkdf};
use hex_literal::hex;
use dioxus::prelude::*;
use dioxus_fullstack::launch::LaunchBuilder;
use dioxus_fullstack::prelude::{server, ServerFnError};
use tera::{Context, Tera};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use ring::hkdf::HKDF_SHA256;
use ring::pbkdf2::PBKDF2_HMAC_SHA512;
use ring::rand::SecureRandom;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::process::exit;
use std::time::Duration;
use serde::{Serialize, Serializer};
#[cfg(feature = "ssr")]
use tokio::signal;

const INFO: [u8; 10] = hex!("f0f1f2f3f4f5f6f7f8f9");
const B32_ENCODING_ALPHABET: base32::Alphabet = base32::Alphabet::RFC4648 {padding: true};

#[derive(Debug, PartialEq)]
struct HkdfMy<T: core::fmt::Debug + PartialEq>(T);

impl hkdf::KeyType for HkdfMy<usize> {
    fn len(&self) -> usize {
        self.0
    }
}

impl From<hkdf::Okm<'_, HkdfMy<usize>>> for HkdfMy<Vec<u8>> {
    fn from(okm: hkdf::Okm<HkdfMy<usize>>) -> Self {
        let mut r = vec![0u8; okm.len().0];
        okm.fill(&mut r).unwrap();
        Self(r)
    }
}


#[derive(Debug, EnumIter)]
enum NcpSecrets {
    NcAioDbPassword,
    NcAioFulltextsearchPw,
    NcAioNcDomain,
    NcAioNextcloudPassword,
    NcAioOnlyofficeSecret,
    NcAioRecordingSecret,
    NcAioRedisPassword,
    NcAioSignalingSecret,
    NcAioTalkInternalSecret,
    NcAioTurnSecret,
    NcpKdkSalt,
    NcpStaticKdkEnc
}


impl NcpSecrets {
    fn as_str(&self) -> &'static str {
        match self {
            NcpSecrets::NcAioDbPassword => "AIO_DATABASE_PASSWORD",
            NcpSecrets::NcAioFulltextsearchPw => "AIO_FULLTEXTSEARCH_PASSWORD",
            NcpSecrets::NcAioNcDomain => "AIO_NEXTCLOUD_DOMAIN",
            NcpSecrets::NcAioNextcloudPassword => "AIO_NEXTCLOUD_PASSWORD",
            NcpSecrets::NcAioOnlyofficeSecret => "AIO_ONLYOFFICE_SECRET",
            NcpSecrets::NcAioRecordingSecret => "AIO_RECORDING_SECRET",
            NcpSecrets::NcAioRedisPassword => "AIO_REDIS_PASSWORD",
            NcpSecrets::NcAioSignalingSecret => "AIO_SIGNALING_SECRET",
            NcpSecrets::NcAioTalkInternalSecret => "AIO_TALK_INTERNAL_SECRET",
            NcpSecrets::NcAioTurnSecret => "AIO_TURN_SECRET",
            NcpSecrets::NcpKdkSalt => "NCP_KEY_DERIVATION_KEY_SALT",
            NcpSecrets::NcpStaticKdkEnc => "NCP_STATIC_KEY_DERIVATION_KEY_ENC"
        }
    }

    fn to_sha256(&self) -> Vec<u8> {
        let hash = digest::digest(&digest::SHA256, self.as_str().as_bytes());
        hash.as_ref().to_vec()
    }
}

#[derive(Debug)]
struct NcpError {
    msg: String
}

impl std::error::Error for NcpError {}

impl fmt::Display for NcpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NcpError: {}", self.msg)
    }
}

impl From<String> for NcpError {
    fn from(value: String) -> Self {
        NcpError{ msg: value }
    }
}

impl From<&str> for NcpError {
    fn from(value: &str) -> Self {
        NcpError{ msg: value.to_string() }
    }
}

impl From<std::io::Error> for NcpError {
    fn from(value: std::io::Error) -> Self {
        NcpError{ msg: format!("{}({})", &value.kind(), value.to_string()) }
    }
}

impl Serialize for NcpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(format!("NCPError({})", self.msg).as_str())
    }
}

fn create_key_from_pass(salt: [u8; 16], pass: &str) -> [u8; 64] {
    let mut derived = [0u8; digest::SHA512_OUTPUT_LEN];
    pbkdf2::derive(PBKDF2_HMAC_SHA512, NonZeroU32::new(100_000).unwrap(), &salt, pass.as_bytes(), &mut derived);
    derived
}

fn generate_salt() -> Result<[u8; 16], NcpError> {
    let rng = ring::rand::SystemRandom::new();
    let mut buf = [0; 16];
    match rng.fill(&mut buf) {
        Err(_) => Err("Failed to fill buffer for salt".into()),
        Ok(_) => Ok(buf)
    }
}
fn derive_key(salt: &[u8], mk: &[u8]) -> Result<Vec<u8>, NcpError> {
    let hdkf_salt = hkdf::Salt::new(HKDF_SHA256, salt);
    match hdkf_salt.extract(mk).expand(&[&INFO], HkdfMy(42)) {
        Ok(myOkm) => {
            let HkdfMy(okm) = myOkm.into();
            Ok(okm)
        },
        Err(e) => Err("Failed to derive okm".into())
    }
}

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

fn render_template(ctx: Context, path_in: PathBuf, path_out: PathBuf) -> Result<(), NcpError> {
    fs::create_dir_all(path_out.parent()
        .ok_or(NcpError::from(format!("Invalid template output path: {path_out:?}")))?)?;
    let mut tera = Tera::default();
    tera.add_template_file(&path_in, Some("default")).map_err(|_| format!("Failed to load template file {path_in:?}"))?;
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .open(&path_out)
        .map_err(|e| e.to_string())?;
    tera.render_to("default", &ctx,  out_file).map_err(|e| format!("Failed to render template to {path_out:?} ({e:?})"))?;
    //unsafe { libc::raise(libc::SIGTERM) }
    // exit(0);
    Ok(())
}

fn print_err<E: std::error::Error>(e: E) -> E {
    eprintln!("{:?}", e);
    e
}

#[server]
async fn activate_ncp(user_pass: String) -> Result<(), ServerFnError>{
    type S = dyn Into<String>;
    let master_salt = generate_salt().map_err(print_err)?;
    let master_key = create_key_from_pass(master_salt, "testpw");
    let derived_salt = generate_salt().unwrap();
    let derived = derive_key(&derived_salt, &master_key);
    println!("master key: {master_key:?}\nmaster key salt: {master_salt:?}\nderived_salt: {derived_salt:?}\nderived: {derived:?}");
    let mut tera_ctx = Context::new();
    tera_ctx.insert(NcpSecrets::NcpKdkSalt.as_str(), &base32::encode(B32_ENCODING_ALPHABET, &master_salt).as_str());
    tera_ctx.insert(NcpSecrets::NcpStaticKdkEnc.as_str(), "");
    for secret in NcpSecrets::iter() {
        tera_ctx.insert(secret.as_str(), &base32::encode(B32_ENCODING_ALPHABET, &derive_key(&secret.to_sha256(), &master_key).map_err(print_err)?));
    }
    tera_ctx.remove(NcpSecrets::NcAioNcDomain.as_str());
    tera_ctx.insert(NcpSecrets::NcAioNcDomain.as_str(), "localhost");
    let secrets_template_base_path = PathBuf::from(env::var("NCP_CONFIG_SOURCE").map_err(print_err)?);
    let secrets_render_base_path = PathBuf::from(env::var("NCP_CONFIG_TARGET").map_err(print_err)?);
    let secrets_template_path = secrets_template_base_path.join("secrets.json");
    let secrets_render_path = secrets_render_base_path.join("secrets.json");
    render_template(tera_ctx.clone(), PathBuf::from(secrets_template_path), PathBuf::from(secrets_render_path)).map_err(print_err)?;
    let aio_secrets_template_path = secrets_template_base_path.join("nextcloud-aio/defaults.env");
    let aio_secrets_render_path = secrets_render_base_path.join("nextcloud-aio/.env");
    render_template(tera_ctx, PathBuf::from(aio_secrets_template_path), PathBuf::from(aio_secrets_render_path)).map_err(print_err)?;
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

pub fn app(cx: Scope) -> Element {
    let mut userpass = use_state(cx, || "".to_string());
    let mut status = use_state(cx, || "".to_string());
    // let eval = use_eval(cx);
    // let nc_status_check = use_future(cx, (), |_| async move {
    //     reqwest::get("https://")
    // });
    cx.render(rsx! {
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
                to_owned![status];
                to_owned![userpass];
                async move {
                    status.set(match activate_ncp(userpass.current().to_string()).await {
                            Err(e) => e.to_string(),
                            Ok(_) => {
                                terminate().await.expect("Failed to stop server");
                                "NCP activated successfully!".to_string()
                            }
                        }
                    );

                }

            },
            "Activate NCP",
        },
        div {
            "{status}",
        }
    })
}
