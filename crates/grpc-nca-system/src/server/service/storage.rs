mod util;

use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use nca_error::NcaError;
use crate::api::{PasswordResponse};
use crate::api::storage_server::Storage;
use crate::crypto::{b32_encode, create_key_from_pass, derive_key};
use crate::server::storage::add_fallback_password_to_encrypted_disks;

pub struct StorageService {
    config: Arc<Mutex<crate::server::config::Config>>,
}

impl StorageService {
    pub fn new(config: Arc<Mutex<crate::server::config::Config>>) -> Self {
        Self { config }
    }
}

#[tonic::async_trait]
impl Storage for StorageService {

    async fn add_disk_encryption_password(&self, request: Request<crate::api::PrimaryPassword>) -> Result<Response<PasswordResponse>, Status> {
        let salt = {
            let cfg = self.config.lock().await;
            match cfg.salt {
                None => return Err(Status::failed_precondition("Instance salt has not been generated yet!")),
                Some(salt) => salt,
            }
        };

        let primary_key = create_key_from_pass(&salt, request.into_inner().value);
        let disk_encryption_password = derive_key(&primary_key, &salt, "NCATOMIC_DISK_ENCRYPTION".to_string())
            .map_err(|e| NcaError::CryptoError(format!("Failed to derive key from password: {e:?}")))?;
        let disk_encryption_password_b32 = b32_encode(&disk_encryption_password);
        add_fallback_password_to_encrypted_disks(disk_encryption_password_b32.clone()).await?;
        Ok(Response::new(PasswordResponse {
            password: disk_encryption_password_b32,
            status: 200
        }))

    }
}
