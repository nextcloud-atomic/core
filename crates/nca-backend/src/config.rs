use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Config {
    pub address: SocketAddr,
    pub caddy_admin_socket: Option<String>,
    pub occ_socket: String,
    pub config_path: String,
}

impl Config {
    pub fn new() -> Config {
        // let journal_grpc_address = std::env::var("JOURNAL_GRPC_ADDRESS").expect("JOURNAL_GRPC_ADDRESS not set");
        let port = std::env::var("PORT").unwrap_or("3000".to_string());
        let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
        let caddy_admin_socket = std::env::var("CADDY_ADMIN_SOCKET").ok();
        let config_path = std::env::var("CONFIG_PATH").unwrap_or("/etc/ncatomic".to_string());
        #[cfg(not(feature = "mock-occ"))]
        let occ_socket = std::env::var("OCC_SERVER_SOCKET")
            .expect("Missing environment variable 'OCC_SERVER_SOCKET'");
        #[cfg(feature = "mock-occ")]
        let occ_socket = "".to_string();
        let address = SocketAddr::from_str(format!("{host}:{port}").as_str())
            .expect(format!("Could not parse address from host ({host}) and port ({port}).").as_str());

        Config {
            address,
            caddy_admin_socket,
            occ_socket,
            config_path,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: SocketAddr::from_str("127.0.0.1:3001").unwrap(), 
            caddy_admin_socket: None, 
            occ_socket: "".to_string(),
            config_path: "/etc/ncatomic".to_string(),
        }
    }
}