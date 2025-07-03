use std::net::SocketAddr;
use std::str::FromStr;
use http::Uri;
use tonic::transport::Channel;
use grpc_common::client::retrieve_grpc_channel;

#[derive(Clone, Debug)]
pub struct Config {
    pub address: SocketAddr,
    pub caddy_admin_socket: Option<String>,
    pub occ_channel: Channel,
    pub nca_system_channel: Channel,
    pub config_path: String,
}

impl Config {
    pub async fn new() -> Config {
        let port = std::env::var("PORT").unwrap_or("3000".to_string());
        let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
        let caddy_admin_socket = std::env::var("CADDY_ADMIN_SOCKET").ok();
        let config_path = std::env::var("CONFIG_PATH").unwrap_or("/etc/ncatomic".to_string());
        let address = SocketAddr::from_str(format!("{host}:{port}").as_str())
            .expect(format!("Could not parse address from host ({host}) and port ({port}).").as_str());

        #[cfg(not(feature = "mock-occ"))]
        let occ_channel = retrieve_grpc_channel(
            "OCC_SERVER_ADDRESS",
            "NCATOMIC_SOCKETS_PATH",
            "/run/ncatomic/",
            "occ.sock",
            "http://occ.nextcloudatomic.local"
        ).await
            .map_err(|e| format!("Failed to get nca system channel: {e:?}"))
            .unwrap();
        #[cfg(feature = "mock-occ")]
        let occ_channel = Channel::builder(Uri::from_static("http://localhost")).connect_lazy();
        
        #[cfg(not(feature = "mock-systemd"))]
        let nca_system_channel = retrieve_grpc_channel(
            "NCA_SYSTEM_ADDRESS",
            "NCATOMIC_SOCKETS_PATH",
            "/run/ncatomic",
            "nca-system.sock",
            "http://system.nextcloudatomic.local"
        ).await
            .map_err(|e| format!("Failed to get nca system channel: {e:?}"))
            .unwrap();
        #[cfg(feature = "mock-systemd")]
        let nca_system_channel = Channel::builder(Uri::from_static("http://localhost")).connect_lazy();

        Config {
            address,
            caddy_admin_socket,
            occ_channel,
            nca_system_channel,
            config_path,
        }
    }
}
