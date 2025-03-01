
use std::collections::HashMap;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use nca_error::NcaError;
use crate::api::{LogMessage, LogFilter, journal_log_stream_server::JournalLogStream};
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Default)]
pub struct JournalLogStreamService {
    user_logs: bool,
    system_logs: bool
}

impl JournalLogStreamService {
    pub fn new(user_logs: bool, system_logs: bool) -> JournalLogStreamService {
        JournalLogStreamService { user_logs, system_logs }
    }
}

#[tonic::async_trait]
impl JournalLogStream for JournalLogStreamService {
    type TailStream = ReceiverStream<Result<LogMessage, Status>>;

    #[cfg(not(feature = "mock"))]
    async fn tail(&self, request: Request<LogFilter>) -> Result<Response<Self::TailStream>, Status> {
        #[cfg(debug_assertions)]
        {
            println!("Received tail request: {:?}", request);
        }

        let filter = request.into_inner();

        let rx = get_log_stream(self.user_logs, self.system_logs, filter.namespace, Some(filter.fields)).await;
        Ok(Response::new(ReceiverStream::new(rx)))
    }
    
    #[cfg(feature = "mock")]
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
                        ("CONTAINER_NAME".to_string(), "mock-container".to_string())
                    ]),
                    message: "hello world".to_string(),
                    namespace: None
                };
                if let Err(e) = tx.blocking_send(Ok(msg)) {
                    panic!("Error while sending message: {e:?}");
                }
                sleep(Duration::from_secs(5));
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub async fn get_log_stream(current_user: bool, system: bool, namespace: Option<String>, filter_fields: Option<HashMap<String, String>>) -> Receiver<Result<LogMessage, Status>> {
    println!("getting log stream");

    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let default_msg = "<no message>".to_string();

    const KEY_UNIT: &str = "_SYSTEMD_UNIT";
    const KEY_MESSAGE: &str = "MESSAGE";

    thread::spawn(move || {
        use systemd::journal::{self, JournalSeek};
        let filter_fields = filter_fields.unwrap_or_default();
        let mut opts = journal::OpenOptions::default()
            .system(system)
            .current_user(current_user)
            .system(system).to_owned();
        let reader_result = match &namespace {
            None => opts
                .all_namespaces(true)
                .open(),
            Some(ns) => opts.open_namespace(ns)
        };
        let mut reader = match reader_result {
            Err(e) => {
                eprintln!("Couldn't open journal: {e:?}");
                return;
            },
            Ok(reader) => reader
        };
        // TODO: Fix filtering by fields and reenable
        for (k, v) in filter_fields.iter() {
            if let Err(e) = reader.match_add(k, v.as_str()) {
                eprintln!("Error adding filter: {e:?}");
            }
        }

        reader.seek(JournalSeek::Tail).expect("Couldn't seek to end of journal");
        if let Err(e) = reader.previous().map_err(|e| NcaError::IOError(e.to_string())) {
            eprintln!("{e:?}");
            return;
        };

        let mut abort = false;
        loop {
            loop {
                match reader.next() {
                    Ok(0) => {
                        // if let Err(e) = tx.blocking_send(Err(Status::cancelled("no more messages found"))) {
                        //     eprintln!("An error occurred while cancelling stream: {e:?}");
                        // }
                        break;
                    },
                    Ok(_) => {},
                    Err(e) => {
                        if let Err(e2) = tx.blocking_send(Err(Status::cancelled("unexpected error"))) {
                            eprintln!("Unexpected error: {e:?}\n Also, an error occurred while cancelling stream: {e2:?}");
                        }
                        eprintln!("Unexpected error: {e:?}");
                        abort = true;
                        break;
                    }
                }

                match reader.await_next_entry(None) {
                    Err(e) => {
                        if let Err(e2) = tx.blocking_send(Err(Status::cancelled("Unexpected error while waiting for journal entries"))) {
                            eprintln!("Unexpected error while waiting for journal entries: {e:?}\nAlso an error occurred while cancelling stream: {e2:?}");
                        }
                        eprintln!("Unexpected error while waiting for journal entries: {e:?}");
                        abort = true;
                        break;
                    },
                    Ok(None) => {
                        break;
                        if let Err(e) = tx.blocking_send(Err(Status::cancelled("Unexpected end of journal"))) {
                            eprintln!("Unexpected end of journal\nAlso, an error occurred while cancelling stream: {e:?}");
                        }
                        eprintln!("Unexpected end of journal");
                        abort = true;
                        break;
                    },
                    Ok(Some(record)) => {
                        // if !filter_fields.iter().all(|(k, v)| {
                        //     match record.get(k) {
                        //         None => false,
                        //         Some(val) => val == v,
                        //     }
                        // }) {
                        //     continue;
                        // }
                        let record_map = record.into_iter().collect::<HashMap<String, String>>();
                        let msg = record_map.get(KEY_MESSAGE).unwrap_or(&default_msg).clone();
                        #[cfg(debug_assertions)]
                        {
                            println!("Sending journal msg: {msg}");
                        }
                        let log_msg = LogMessage {
                            fields: record_map,
                            message: msg,
                            namespace: namespace.clone(),
                        };
                        if let Err(e) = tx.blocking_send(Ok(log_msg)) {
                            eprintln!("Receiver has gone away, exiting ... ({e:?})");
                            abort = true;
                            break;
                        }
                    }
                }

            }
            if abort {
                break;
            }
            if let Err(e) = reader.wait(None) {
                eprintln!("Unexpected error while waiting for journal entries: {e:?}");
            }
        }
    });

    rx
}


#[cfg(test)]
mod tests {
    use futures_util::stream::StreamExt;
    use hyper_util::rt::TokioIo;
    use tonic::transport::{Endpoint, Server, Uri};
    use tower::service_fn;
    use crate::api::journal_log_stream_client::JournalLogStreamClient;
    use crate::api::journal_log_stream_server::JournalLogStreamServer;
    use crate::api::LogFilter;
    use crate::journal_stream::get_log_stream;
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