use tonic::{Response, Status};
use nca_system_api::systemd::api::set_systemd_credential;
use crate::api::StatusResponse;

pub(crate) async fn set_systemd_credential_at_path(value: String, path: String, name: Option<String>) -> Result<Response<StatusResponse>, Status> {
    match set_systemd_credential(value, path, name, false).await {
        Err(e) => Err(e.into()),
        Ok(out) => {
            println!("{out}");
            Ok(Response::new(StatusResponse::default()))
        }
    }
}