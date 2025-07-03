use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use nca_system_api::systemd::api::start_service;
use crate::api::{Empty, StatusResponse};
use crate::api::system_server::System;

pub struct SystemService {
    config: Arc<Mutex<crate::server::config::Config>>,
}

impl SystemService {
    pub fn new(config: Arc<Mutex<crate::server::config::Config>>) -> Self {
        Self { config }
    }
}

#[tonic::async_trait]
impl System for SystemService {
    async fn unlock_from_systemd_credentials(&self, request: Request<Empty>) -> Result<Response<StatusResponse>, Status> {
        start_service("nca-unlock.service".to_string()).await?;
        Ok(Response::new(StatusResponse {
            status: 200,
            status_text: "Successfully unlocked system from systemd credentials".to_string(),
        }))
        
    }
}