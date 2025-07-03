mod backup;
pub mod credentials_config;

use std::fs;
use std::path::{PathBuf};
use nca_error::NcaError;
use crate::api::credentials_server::Credentials;
use crate::crypto::{try_parse_salt, Salt};
use crate::server::config::backup::BackupConfig;
use crate::server::config::credentials_config::CredentialsConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub config_path: String,
    pub salt: Option<Salt>,
    pub backup: Option<BackupConfig>,
    pub setup_complete: bool,
    pub credentials_config: Option<CredentialsConfig>,
}

impl Config {
    pub fn new() -> Result<Self, NcaError> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or("/etc/ncatomic".to_string());
        let (
            salt,
        ) = match std::env::var("CREDENTIALS_DIRECTORY") {
            Err(_) => {
                println!("no credentials configured yet.");
                (None,)
            },
            Ok(credentials_dir) => {
                let credentials_base = PathBuf::from(credentials_dir);
                let salt_path = credentials_base.join("ncatomic_salt.txt");
                let salt = match salt_path.exists() {
                    false => None, 
                    true => match fs::read_to_string(&salt_path) {
                        Err(_) => None,
                        Ok(salt_str) => {
                            Some(
                                try_parse_salt(&salt_str)
                                    .map_err(|e| NcaError::new_crypto_error(
                                        format!("Failed to load salt from credentials: {e:?}"))
                                    )?
                            )
                        },
                    }
                };
                (salt,)
            }
        };

        Ok(Self {
            config_path,
            salt,
            backup: None,
            setup_complete: salt.is_some(),
            credentials_config: None,
        })
    }
}