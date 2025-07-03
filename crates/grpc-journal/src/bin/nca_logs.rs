use std::collections::HashMap;
use std::process::exit;
use grpc_journal::client::stream_logs;
use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;
use futures_util::StreamExt;
use grpc_journal::api::{LogFilter};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    namespace: Option<String>,
    filters: Vec<String>,
}


#[tokio::main]
async fn main() {
    
    let cli = Cli::parse();
    if let Some(invalid) = cli.filters.iter().find(|f| !f.contains("=")) {
        let mut cmd = Cli::command();
        cmd
            .error(
                ErrorKind::InvalidValue,
                format!("--filter arguments must container a '=' (but '--filter {invalid}' does not)"))
            .exit();
    }
    let filters: HashMap<String, String> = cli.filters.into_iter()
        .map(|f| {
            let (k, v) = f.split_once("=").unwrap();
            (k.to_string(), v.to_string())
        })
        .collect();
    let namespace = cli.namespace;

    let socket_path = std::env::var("GRPC_JOURNAL_SOCKET_PATH")
        .expect("$GRPC_JOURNAL_SOCKET is not set");

    let stream_result = stream_logs(socket_path, LogFilter {
        namespace,
        fields: filters
    }).await;
    
    match stream_result {
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        },
        Ok(stream) => {
            stream.into_inner().for_each(|log| async {
                match log {
                    Err(e) => {
                        match e.code() {
                            tonic::Code::Ok => exit(0),
                            tonic::Code::Cancelled => {
                                eprintln!("Stream cancelled.");
                                exit(1)
                            },
                            tonic::Code::Aborted => {
                                eprintln!("Stream was aborted: {}", e.message());
                                exit(2);
                            },
                            _ => {
                                eprintln!("Error receiving message ({}): {}", e.code().to_string(), e.message());
                                exit(3);
                            }
                        };
                    },
                    Ok(msg) => {
                        println!("{}", msg.message);
                    }
                }
            }).await;
        }
    };
    
}