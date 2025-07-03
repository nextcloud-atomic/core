use std::fs;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use nca_error::NcaError;
use crate::api::nextcloud_server::Nextcloud;
use crate::api::{CommandOutput, Empty, NextcloudConfig, StatusResponse};
use crate::server::util::set_systemd_credential_at_path;

pub struct NextCloudService {
    config: Arc<Mutex<crate::server::config::Config>>,
}

impl NextCloudService {
    pub fn new(config: Arc<Mutex<crate::server::config::Config>>) -> Self {
        Self {
            config
        }
    }
}

#[tonic::async_trait]
impl Nextcloud for NextCloudService {
    async fn update_config(&self, request: Request<NextcloudConfig>) -> Result<Response<NextcloudConfig>, Status> {
        let config_path = {
            self.config.lock().await.config_path.clone()
        };
        
        let nc_cfg = request.into_inner();
        if let Some(domain) = &nc_cfg.domain {
            set_systemd_credential_at_path(
                domain.clone(),
                format!("{config_path}/nc-aio/credentials/domain.txt"),
                Some("nextcloud_domain.txt".to_string()),
            ).await
                .map_err(|e| NcaError::new_server_config_error(
                    format!("Failed to set nextcloud domain: {e:?}")))?;
        }
        if let Some(admin_password) = &nc_cfg.admin_password {
            set_systemd_credential_at_path(
                admin_password.clone(),
                format!("{config_path}/nc-aio/credentials/admin_password.txt"),
                Some("nextcloud_admin_password.txt".to_string()),
            ).await
                .map_err(|e| NcaError::new_server_config_error(
                    format!("Failed to set nextcloud admin password: {e:?}")))?;
        }
        
        // TODO: Load actual values
        Ok(Response::new(NextcloudConfig {
            domain: nc_cfg.domain,
            admin_password: nc_cfg.admin_password,
        }))
    }

    async fn hard_reset(&self, _request: Request<Empty>) -> Result<Response<CommandOutput>, Status> {
        println!("Resetting nextcloud-all-in-one and deleting its data");
        let proc = std::process::Command::new("su")
            .args(["-l", "-s", "/usr/bin/bash", "-c",
                "/usr/bin/podman podman system reset --force",
                "aio"
            ])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(NcaError::new_io_error)?;
        let out = proc.wait_with_output()
            .map_err(NcaError::new_io_error)?;
        let stdout = String::from_utf8(out.stdout).unwrap_or("failed to decode".to_string());
        let stderr = String::from_utf8(out.stderr).unwrap_or("failed to decode".to_string());
        println!("stdout: {stdout}");
        println!("stderr: {stdout}");
        let nc_path = "/var/data/ncatomic/nc-aio/nextcloud/";
        let nc_files_path = "/var/data/ncatomic/nc-aio/nc-files/";
        if fs::exists(nc_path)? {
            for f in fs::read_dir(nc_path).map_err(NcaError::new_io_error)? {
                let path = f.map_err(NcaError::new_io_error)?.path();
                if path.is_dir() {
                    fs::remove_dir_all(path).map_err(NcaError::new_io_error)?;
                } else {
                    fs::remove_file(path).map_err(NcaError::new_io_error)?;
                }
            }
        }
        if fs::exists(nc_files_path)? {
            for f in fs::read_dir(nc_path).map_err(NcaError::new_io_error)? {
                let path = f.map_err(NcaError::new_io_error)?.path();
                if path.is_dir() {
                    fs::remove_dir_all(path).map_err(NcaError::new_io_error)?;
                } else {
                    fs::remove_file(path).map_err(NcaError::new_io_error)?;
                }
            }
        }
        match out.status.success() {
            true => {
                Ok(Response::new(CommandOutput{
                    rc: out.status.code(),
                    stdout: Some(stdout),
                    stderr: Some(stderr)
                }))
            },
            false => {
                Err(Status::unknown(format!("{stdout}\n---\n{stderr}")))
            }
        }
    }
}