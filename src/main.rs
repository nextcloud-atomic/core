
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;

use tokio_stream::{Stream, StreamExt};
use tonic::codegen::http::HeaderName;
use tonic::transport::Server;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};
use grpc_journal::api::journal_log_stream_server::JournalLogStreamServer;
use grpc_journal::journal_stream::JournalLogStreamService;

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [HeaderName; 3] = [
    HeaderName::from_static("grpc-status"),
    HeaderName::from_static("grpc-message"),
    HeaderName::from_static("grpc-status-details-bin"),
];

async fn run_logstream_backend(user_logs: bool, system_logs: bool) -> Result<(), String> {
    let service = JournalLogStreamService::new(user_logs, system_logs);
    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::new()
            .allow_credentials(true)
            .max_age(DEFAULT_MAX_AGE)
            .expose_headers(ExposeHeaders::from(DEFAULT_EXPOSED_HEADERS)))
        .add_service(tonic_web::enable(JournalLogStreamServer::new(service)))
        .serve("0.0.0.0:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_logstream_backend(false, true).await?;
    Ok(())
}
