
use tokio_stream::wrappers::ReceiverStream;
use std::thread;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use tonic::{Request, Response, Status};
use crate::api::{LogMessage, LogFilter, journal_log_stream_server::JournalLogStream};

#[derive(Debug, Default)]
pub struct JournalLogStreamService {
    user_logs: bool,
    system_logs: bool
}

impl JournalLogStreamService {
    pub fn new(user_logs: bool, system_logs: bool) -> Self {
        JournalLogStreamService{user_logs, system_logs}
    }
}
#[tonic::async_trait]
impl JournalLogStream for JournalLogStreamService {
    type TailStream = ReceiverStream<Result<LogMessage, Status>>;

    async fn tail(&self, request: Request<LogFilter>) -> Result<Response<Self::TailStream>, Status> {
        #[cfg(debug_assertions)]
        {
            println!("grpc::JournalLogStream: Received tail request: {:?}", request);
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<LogMessage, Status>>(100);
        thread::spawn(move || {
            loop {
                let msg = LogMessage {
                    fields: HashMap::from([
                        ("foo".to_string(), "bar".to_string()), 
                        ("_MESSAGE".to_string(), "hello world".to_string()),
                        ("_SYSTEMD_UNIT".to_string(), "mock.service".to_string()),
                        ("_CONTAINER_NAME".to_string(), "mock-container".to_string())
                    ]),
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
