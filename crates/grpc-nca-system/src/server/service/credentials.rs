use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use nca_error::NcaError;
// use crate::api:;
use crate::api::{CredentialsInitResponse, Empty, StatusResponse};
use nca_system_api::systemd::api::set_systemd_credential;
use crate::api::credentials_server::Credentials;
use crate::crypto::{b32_encode, create_key_from_pass, derive_key, generate_salt, Salt};
use crate::server::config::credentials_config::CredentialsConfig;
use crate::server::storage::add_fallback_password_to_encrypted_disks;
use crate::server::util::set_systemd_credential_at_path;

pub struct CredentialsService {
    config: Arc<Mutex<crate::server::config::Config>>,
}

impl CredentialsService {
    pub fn new(config: Arc<Mutex<crate::server::config::Config>>) -> Self {
        Self { config }
    }

    async fn ensure_salt(&self) -> Result<Salt, NcaError> {
        let salt = {
            let cfg = self.config.lock().await;
            cfg.salt
        };
        match salt {
            Some(salt) => Ok(salt),
            None => {
                println!("generating salt ...");
                let salt = generate_salt();
                let salt_b32 = b32_encode(&salt);
                let config_path = {
                    self.config.lock().await.config_path.clone()
                };
                set_systemd_credential(
                    salt_b32,
                    format!("{config_path}/credentials/salt.txt"),
                    Some("ncatomic_salt.txt".to_string()),
                    false
                ).await?;
                let mut cfg = self.config.lock().await;
                cfg.salt = Some(salt);
                Ok(salt)
            }
        }
    }
}

#[tonic::async_trait]
impl Credentials for CredentialsService {
    async fn set_nextcloud_admin_password(&self, request: Request<crate::api::PrimaryPassword>) -> Result<Response<StatusResponse>, Status> {
        let config_path = {
            self.config.lock().await.config_path.clone()
        };
        set_systemd_credential_at_path(
            request.into_inner().value,
            format!("{config_path}/nc-aio/credentials/nc_password.txt"),
            None
        ).await
    }

    async fn set_backup_password(&self, request: Request<crate::api::PrimaryPassword>) -> Result<Response<StatusResponse>, Status> {
        let config_path = {
            self.config.lock().await.config_path.clone()
        };
        set_systemd_credential_at_path(
            request.into_inner().value,
            format!("{config_path}/credentials/backup_password.txt"),
            Some("ncatomic_backup_password.txt".to_string())
        ).await
    }

    async fn generate_salt(&self, _request: Request<Empty>) -> Result<Response<StatusResponse>, Status> {
        let salt = {
            self.config.lock().await.salt
        };
        if salt.is_some() {
            return Err(Status::failed_precondition("salt was already set"))
        };
        self.ensure_salt().await?;
        Ok(Response::new(StatusResponse::default()))
    }

    async fn initialize_credentials(&self, request: Request<crate::api::PrimaryPassword>) -> Result<Response<CredentialsInitResponse>, Status> {

        let cfg = {
            self.config.lock().await.clone()
        };
        if cfg.setup_complete {
            return Err(Status::failed_precondition("Instance already initialized"))
        };
        
        let salt = self.ensure_salt().await?;
        
        let primary_password = request.into_inner().value;
        let salt_b32 = b32_encode(&salt);
        let primary_key = create_key_from_pass(&salt, primary_password.clone());
        let disk_encryption_password = derive_key(&primary_key, &salt, "NCA_DISK_ENCRYPTION".to_string())
            .map_err(|e| NcaError::CryptoError(format!("Failed to derive key from password: {e:?}")))?;
        let disk_encryption_password_b32 = b32_encode(&disk_encryption_password);
        let backup_password = derive_key(&primary_key, &salt, "NCA_BACKUP_ENCRYPTION".to_string())
            .map_err(|e| NcaError::CryptoError(format!("Failed to derive key from password: {e:?}")))?;
        let backup_password_b32 = b32_encode(&backup_password);

        {
            let mut cfg = self.config.lock().await;
            cfg.credentials_config = Some(CredentialsConfig {
                disk_encryption_password: disk_encryption_password_b32.clone(),
                backup_password: backup_password_b32.clone()
            });
        }
        
        Ok(Response::new(CredentialsInitResponse{
            backup_password: backup_password_b32,
            disk_encryption_recovery_password: disk_encryption_password_b32,
            salt: salt_b32,
        }))
    }

    async fn complete_setup(&self, _request: Request<Empty>) -> Result<Response<StatusResponse>, Status> {
        let (config_path, salt, backup_password_b32, disk_encryption_password_b32) = {
            let cfg = self.config.lock().await;
            let salt = match &cfg.salt {
                None => return Err(Status::failed_precondition("salt not set")),
                Some(salt) => salt,
            };
            let credentials = match &cfg.credentials_config {
                None => return Err(Status::failed_precondition("credentials not set")),
                Some(creds) => creds
            };
            (
                &cfg.config_path.clone(), 
                salt.to_owned(), 
                credentials.backup_password.clone(),
                credentials.disk_encryption_password.clone()
            )
        };
        
        let salt_b32 = b32_encode(&salt);

        add_fallback_password_to_encrypted_disks(disk_encryption_password_b32).await?;

        set_systemd_credential_at_path(
            salt_b32,
            format!("{config_path}/credentials/salt.txt"),
            Some("ncatomic_salt.txt".to_string())
        ).await?;

        set_systemd_credential_at_path(
            backup_password_b32,
            format!("{config_path}/credentials/backup_password.txt"),
            Some("ncatomic_backup_password.txt".to_string())
        ).await?;
        
        Ok(Response::new(StatusResponse{
            status: 200,
            status_text: "Setup completed successfully".to_string(),
        }))
    }
}
    