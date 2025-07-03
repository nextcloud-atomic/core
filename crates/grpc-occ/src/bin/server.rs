use std::fs;
use std::path::PathBuf;
use tokio::net::UnixListener;
use tonic::codegen::tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use grpc_common::server::serve_socket_tonic;
use grpc_occ::api::occ_server::OccServer;
use grpc_occ::occ::server::OccService;

#[tokio::main]
async fn main() -> Result<(), String> {
    // let (stop_trigger, stop_listener) = triggered::trigger();
    let service = OccService::new(
        "podman".to_string(),
        "nc-aio_nextcloud-aio-nextcloud_1".to_string());

    let socket_path = PathBuf::from(std::env::var("OCC_SOCKET_PATH")
        .expect("OCC_SOCKET_PATH must be set"));
    if socket_path.exists() {
        fs::remove_file(&socket_path).map_err(|e| format!("Failed to remove socket file: {e:?}"))?;
    }
    let uds = UnixListener::bind(&socket_path)
        .map_err(|e| format!("Failed to bind to socket: {socket_path:?}; because: {e:?}"))?;
    let stream = UnixListenerStream::new(uds);

    let grpc = Server::builder()
        .add_service(OccServer::new(service));
    
    serve_socket_tonic(stream, grpc, None).await
        .map_err(|e| format!("An error occurred while running occ server: {e:?}"))?;
    Ok(())
}
