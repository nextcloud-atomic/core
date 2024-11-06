use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use dioxus::prelude::{server, ServerFnError};
use caddy::CaddyClient;

#[cfg(feature = "server")]
async fn caddy_enable_activation_page() -> anyhow::Result<()> {
    let caddy_cli = CaddyClient::new(&env::var("CADDY_ADMIN_SOCKET")?)?;
    let mut f = File::options().read(true).open("/resource/caddy/default_ncatomic_activation.json")?;
    let mut cfg = String::new();
    f.read_to_string(&mut cfg)?;
    caddy_cli.set_caddy_servers(cfg).await?;
    Ok(())
}

#[cfg(feature = "server")]
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

#[tokio::main]
async fn main() {
    let config_path = PathBuf::from(env::var("NCA_CONFIG_TARGET")
        .expect("NCA_CONFIG_TARGET must be set"));
    // i
    if config_path.join("ncatomic.json").exists() {
        caddy_enable_nextcloud().await.expect("Failed to enable nextcloud page");
    } else {
        caddy_enable_activation_page().await.expect("Failed to configure caddy");
    }
}