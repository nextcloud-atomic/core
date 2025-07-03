use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use grpc_common::server::{serve_systemd_socket_tonic, SocketSelectionStrategy};
use grpc_nca_system::api::credentials_server::CredentialsServer;
use grpc_nca_system::api::nextcloud_server::NextcloudServer;
use grpc_nca_system::api::storage_server::StorageServer;
use grpc_nca_system::api::system_server::SystemServer;
use grpc_nca_system::server::config::Config;
use grpc_nca_system::server::service::credentials::CredentialsService;
use grpc_nca_system::server::service::nextcloud::NextCloudService;
use grpc_nca_system::server::service::storage::StorageService;
use grpc_nca_system::server::service::system::SystemService;

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = Arc::new(Mutex::new(Config::new().map_err(|e| e.to_string())?));
    let config_service = CredentialsService::new(config.clone());
    let storage_service = StorageService::new(config.clone());
    let nextcloud_service = NextCloudService::new(config.clone());
    let system_service = SystemService::new(config.clone());

    let grpc = Server::builder()
        .add_service(CredentialsServer::new(config_service))
        .add_service(StorageServer::new(storage_service))
        .add_service(NextcloudServer::new(nextcloud_service))
        .add_service(SystemServer::new(system_service));
    

    if let Err(e) = serve_systemd_socket_tonic(SocketSelectionStrategy::First, grpc, None).await {
        let msg = format!("Error while running nca-system server: {e:?}");
        eprintln!("{msg}");
        return Err(msg)
    };

    println!("nca-system complete");
    Ok(())
}
