
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;

use tokio_stream::{Stream, StreamExt};
use tonic::codegen::http::HeaderName;
use tonic::transport::Server;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};
use grpc_journal::api::journal_log_stream_server::JournalLogStreamServer;
use grpc_journal::JournalLogStreamService;

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
            // TODO: Enable CORS
            .allow_origin(AllowOrigin::mirror_request())
            .allow_headers(AllowHeaders::mirror_request())
            .allow_methods(AllowMethods::mirror_request())
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

#[cfg(test)]
mod tests {
    use hyper_util::rt::TokioIo;
    use tonic::transport::{Endpoint, Uri};
    use tower::service_fn;
    use grpc_journal::get_log_stream;
    use grpc_journal::api::journal_log_stream_client::JournalLogStreamClient;
    use grpc_journal::api::LogFilter;
    use super::*;

    #[tokio::test]
    async fn test_get_log_stream() {
        let mut rx = get_log_stream(true, true, None, None).await;
        let default_msg = String::from("<no message>");
        let empty_string = String::new();
        for _ in 0..4 {
            let msg = rx.recv().await.unwrap().unwrap();
            let msg_text = msg.message;
            let systemd_unit = msg.fields.get("_SYSTEMD_UNIT").unwrap_or(&empty_string);
            let user_unit = msg.fields.get("_SYSTEMD_USER_UNIT").unwrap_or(&empty_string);
            //record.clone().into_keys().reduce(|acc, b| format!("{acc}, {b}")).unwrap()
            println!("[{user_unit}{systemd_unit}] {msg_text}");
        }
    }

    #[tokio::test]
    async fn test_consume_log_stream_grpc() {
        let (client, server) = tokio::io::duplex(1024);
        let service = JournalLogStreamService::new(true, true);
        tokio::spawn(async move {
            Server::builder()
                .add_service(JournalLogStreamServer::new(service))
                .serve_with_incoming(tokio_stream::once(Ok::<_, std::io::Error>(server)))
                .await
                .unwrap();
        });

        let mut client = Some(client);
        let channel = Endpoint::try_from("http://[::]:50051").unwrap()
            .connect_with_connector(service_fn(move |_: Uri| {
                let client = client.take();
                async move {
                    if let Some(client) = client {
                        Ok(TokioIo::new(client))
                    } else {
                        Err(std::io::Error::other("Client already taken"))
                    }
                }
            }))
            .await
            .unwrap();
        let mut client = JournalLogStreamClient::new(channel);
        let request = tonic::Request::new(LogFilter {
            fields: HashMap::from([]),
            namespace: None,
        });
        let stream = client.tail(request).await.unwrap().into_inner();

        let mut stream = stream.take(5);

        let empty_string = String::new();

        while let Some(result) = stream.next().await {
            let record = result.unwrap();
            let systemd_unit = record.fields.get("_SYSTEMD_UNIT").unwrap_or(&empty_string);
            let user_unit = record.fields.get("_SYSTEMD_USER_UNIT").unwrap_or(&empty_string);

            println!("[{systemd_unit}{user_unit}] {:?}", record.message);
        }

    }

}