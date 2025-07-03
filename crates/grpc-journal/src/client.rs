use std::path::PathBuf;
use tonic::{Request, Response, Streaming};
use grpc_common::client::get_socket_channel;
use nca_error::NcaError;
use crate::api::journal_log_stream_client::JournalLogStreamClient;
use crate::api::LogFilter;

pub async fn stream_logs(socket_path: String, filter: LogFilter) -> Result<Response<Streaming<crate::api::LogMessage>>, NcaError> {
    let channel = get_socket_channel(
        PathBuf::from(socket_path),
        "http://occ.nextcloudatomic.local".to_string()
    ).await?;
    
    let mut client = JournalLogStreamClient::new(channel);
    
    client.tail(Request::new(filter)).await
        .map_err(|status| NcaError::new_io_error(status.message()))
    
}
