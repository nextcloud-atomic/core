use std::collections::HashMap;
use std::{fmt, thread};
use std::fmt::Display;
use std::net::ToSocketAddrs;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use api::{LogMessage, LogFilter, journal_log_stream_server::JournalLogStream};

pub mod api {
    tonic::include_proto!("api");
}

#[derive(Debug)]
pub enum NcAtomicError {
    Generic(String),
    Unexpected(String),
    WeakPassword(usize, usize),
    MissingConfig(String),
    InvalidPath(String, String),
    ServerConfiguration(String),
    NotActivated(String),
    IOError(String),
    CryptoError(String),
}

impl fmt::Display for NcAtomicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cause: String = self.get_cause();
        let error_prefix = match *self {
            NcAtomicError::Generic(_) => "Generic Err",
            NcAtomicError::Unexpected(_) => "Unexpected Err",
            NcAtomicError::WeakPassword(_, _) => "WeakPassword Err",
            NcAtomicError::MissingConfig(_) => "Missing Config Err",
            NcAtomicError::InvalidPath(_, _) => "Invalid Path Err",
            NcAtomicError::ServerConfiguration(_) => "Configuration Err",
            NcAtomicError::NotActivated(_) => "Not Activated Err",
            NcAtomicError::CryptoError(_) => "Crypto Err",
            NcAtomicError::IOError(_) => "Input/Output Err",
        };
        write!(f, "{}: {}", error_prefix, cause)
    }
}

impl NcAtomicError {
    fn get_cause(&self) -> String {
        match self {
            NcAtomicError::Generic(cause) => cause.to_string(),
            NcAtomicError::Unexpected(cause) => cause.to_string(),
            NcAtomicError::ServerConfiguration(cause) => cause.to_string(),
            NcAtomicError::NotActivated(cause) => cause.to_string(),
            NcAtomicError::IOError(cause) => cause.to_string(),
            NcAtomicError::CryptoError(cause) => cause.to_string(),
            NcAtomicError::WeakPassword(length, minlength) =>
                format!(
                    "WeakPassword Err: The password needs to be of a minimum length of {}\
                    (only was {} characters long)",
                    minlength, length
                ).to_string(),
            NcAtomicError::MissingConfig(config) =>
                format!("Missing config: {}", config).to_string(),
            NcAtomicError::InvalidPath(path, msg) =>
                format!("The path '{}' is invalid. {}", path, msg).to_string(),
        }
    }

    pub fn new_server_config_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::ServerConfiguration(s.to_string())
    }

    pub fn new_unexpected_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::Unexpected(s.to_string())
    }

    pub fn new_io_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::IOError(s.to_string())
    }

    pub fn new_crypto_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::CryptoError(s.to_string())
    }

    pub fn new_missing_config_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::MissingConfig(s.to_string())
    }
}

impl From<NcAtomicError> for Status {
    fn from(value: NcAtomicError) -> Self {

        match value {
            NcAtomicError::Generic(_) => Status::internal(value.to_string()),
            NcAtomicError::Unexpected(_) => Status::unknown(value.to_string()),
            NcAtomicError::WeakPassword(_, _) => Status::invalid_argument(value.to_string()),
            NcAtomicError::MissingConfig(_) => Status::invalid_argument(value.to_string()),
            NcAtomicError::InvalidPath(_, _) => Status::invalid_argument(value.to_string()),
            NcAtomicError::ServerConfiguration(_) => Status::internal(value.to_string()),
            NcAtomicError::NotActivated(_) => Status::failed_precondition(value.to_string()),
            NcAtomicError::IOError(_) => Status::internal(value.to_string()),
            NcAtomicError::CryptoError(_) => Status::internal(value.to_string()),
        }
    }
}

#[derive(Debug, Default)]
pub struct JournalLogStreamService {
    user_logs: bool,
    system_logs: bool
}

impl JournalLogStreamService {
    pub fn new(user_logs: bool, system_logs: bool) -> JournalLogStreamService {
        JournalLogStreamService{ user_logs, system_logs }
    }
}

#[tonic::async_trait]
impl JournalLogStream for JournalLogStreamService {
    type TailStream = ReceiverStream<Result<LogMessage, Status>>;

    async fn tail(&self, request: Request<LogFilter>) -> Result<Response<Self::TailStream>, Status> {
        #[cfg(debug_assertions)]
        {
            println!("Received tail request: {:?}", request);
        }

        let filter = request.into_inner();

        let rx = get_log_stream(self.user_logs, self.system_logs, filter.namespace, Some(filter.fields)).await;
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
        for (k, v) in filter_fields.iter() {
            if let Err(e) = reader.match_add(k, v.as_bytes()) {
                eprintln!("Error adding filter: {e:?}");
            }
        }

        reader.seek(JournalSeek::Tail).expect("Couldn't seek to end of journal");
        if let Err(e) = reader.previous().map_err(|e| NcAtomicError::IOError(e.to_string())) {
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

