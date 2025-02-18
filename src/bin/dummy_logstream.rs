use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tonic::codegen::http::HeaderName;
use tonic::transport::Server;
use tonic_web::{CorsGrpcWeb, GrpcWebLayer};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};

pub mod api {
    tonic::include_proto!("api");
}
use api::{LogMessage, LogFilter, journal_log_stream_server::JournalLogStreamServer, journal_log_stream_server::JournalLogStream};
use once_cell::sync::Lazy;

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [HeaderName; 3] = [
    HeaderName::from_static("grpc-status"),
    HeaderName::from_static("grpc-message"),
    HeaderName::from_static("grpc-status-details-bin"),
];

// static GRPC_WEB_CORS: Lazy<CorsLayer> = Lazy::new(|| {
//     CorsLayer::new()
//         .allow_origin(AllowOrigin::mirror_request())
//         .allow_headers(AllowHeaders::mirror_request())
//         .allow_methods(AllowMethods::mirror_request())
//         .allow_credentials(true)
//         .max_age(DEFAULT_MAX_AGE)
//         .expose_headers(ExposeHeaders::from(DEFAULT_EXPOSED_HEADERS))
// });

#[derive(Debug, Default)]
struct JournalLogStreamService {
    user_logs: bool,
    system_logs: bool
}
#[tonic::async_trait]
impl JournalLogStream for JournalLogStreamService {
    type TailStream = ReceiverStream<Result<LogMessage, Status>>;

    async fn tail(&self, request: Request<LogFilter>) -> Result<Response<Self::TailStream>, Status> {
        #[cfg(debug_assertions)]
        {
            println!("Received tail request: {:?}", request);
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<LogMessage, Status>>(100);
        thread::spawn(move || {
            loop {
                let msg = LogMessage {
                    fields: HashMap::from([("foo".to_string(), "bar".to_string()), ("_MESSAGE".to_string(), "hello world".to_string())]),
                    message: "hello world".to_string(),
                    namespace: None
                };
                tx.blocking_send(Ok(msg)).unwrap();
                sleep(Duration::from_secs(5));
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

async fn run_logstream_backend(user_logs: bool, system_logs: bool) -> Result<(), String> {
    let service = JournalLogStreamService {user_logs, system_logs};
    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_headers(AllowHeaders::mirror_request())
                .allow_methods(AllowMethods::mirror_request())
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(ExposeHeaders::from(DEFAULT_EXPOSED_HEADERS)))
        .add_service(tonic_web::enable(JournalLogStreamServer::new(service)))
        .serve("127.0.0.1:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_logstream_backend(false, true).await?;
    Ok(())
}
