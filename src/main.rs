#![allow(non_snake_case)]
use hkdf::Hkdf;
use sha2::Sha256;
use bcrypt::{BcryptError, DEFAULT_COST, hash_with_salt, Version};
use hex_literal::hex;
use rand::random;
use dioxus::prelude::*;
use dioxus_fullstack::launch::LaunchBuilder;
use dioxus_fullstack::prelude::{server, ServerFnError};


const INFO: [u8; 10] = hex!("f0f1f2f3f4f5f6f7f8f9");

fn create_master_key(salt: [u8; 16], pass: &str) -> Result<String, BcryptError> {
    let result = hash_with_salt(pass, DEFAULT_COST, salt)?;
    Ok(result.format_for_version(Version::TwoB))
}

fn generate_salt() -> [u8; 16] {
    let ar: [u8; 16] = random();
    return ar;
}
fn derive_key(salt: &[u8], mk: &[u8]) -> Result<[u8; 42], hkdf::InvalidLength> {
    let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &mk);
    let mut okm = [0u8; 42];
    hk.expand(&INFO, &mut okm)?;
    Ok(okm)
}

fn main() {
    LaunchBuilder::new(app).launch();
}

#[server]
async fn activate_ncp(user_pass: String) -> Result<(), ServerFnError>{
    let master_salt = generate_salt();
    let master_key = create_master_key(master_salt, "testpw").map_err(ServerFnError::from)?;
    let derived_salt = generate_salt();
    let derived = derive_key(&derived_salt, master_key.as_bytes());
    let mk_bytes = master_key.as_bytes();
    println!("master key: {mk_bytes:?}\nmaster key salt: {master_salt:?}\nderived_salt: {derived_salt:?}\nderived: {derived:?}");
    Ok(())
    //    .expect("Failed to create master key from password");
}

pub fn app(cx: Scope) -> Element {
    let mut status = use_state(cx, || "");
    cx.render(rsx! {
        div {
            "Set the NCP master password:",
        },
        form {
            onsubmit: move |event| {
                to_owned![status];
                let userpass = event.data.values["userpass"][0].to_string();
                async move {
                    if let Ok(_) = activate_ncp(userpass).await {
                        status.set("NCP activated successfully!");
                    }
                }
            },
            input {
                name: "userpass",
            },
            button {
                r#type: "submit",
                "Activate NCP",
            }
        },
        div {
            "{status}",
        }
    })
}
