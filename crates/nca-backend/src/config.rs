use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Config {
    pub address: SocketAddr,
}

impl Config {
    pub fn new() -> Config {
        // let journal_grpc_address = std::env::var("JOURNAL_GRPC_ADDRESS").expect("JOURNAL_GRPC_ADDRESS not set");
        let port = std::env::var("PORT").unwrap_or("3000".to_string());
        let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
        let address = SocketAddr::from_str(format!("{host}:{port}").as_str())
            .expect(format!("Could not parse address from host ({host}) and port ({port}).").as_str());

        Config {
            address,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {address: SocketAddr::from_str("127.0.0.1:3001").unwrap()}
    }
}