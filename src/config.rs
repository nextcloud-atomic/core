use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use crate::crypto::{Crypto, CryptoValueProvider};
use crate::secrets::{DerivedSecret};
use crate::error::NcpError;


#[serde_inline_default]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcAioConfig {
    #[serde_inline_default(DerivedSecret::from("AIO_DATABASE_PASSWORD"))]
    db_password: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_FULLTEXTSEARCH_PASSWORD"))]
    fulltextsearch_pw: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_NEXTCLOUD_PASSWORD"))]
    nc_password: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_ONLYOFFICE_SECRET"))]
    onlyoffice_secret: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_RECORDING_SECRET"))]
    recording_secret: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_REDIS_PASSWORD"))]
    redis_password: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_SIGNALING_SECRET"))]
    signaling_secret: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_TALK_INTERNAL_SECRET"))]
    talk_internal_secret: DerivedSecret,
    #[serde_inline_default(DerivedSecret::from("AIO_TURN_SECRET"))]
    turn_secret: DerivedSecret,

    #[serde_inline_default(String::from("nextcloudpi.local"))]
    nc_domain: String,
    #[serde_inline_default(false)]
    onlyoffice_enabled: bool,
    #[serde_inline_default(false)]
    collabora_enabled: bool,
    #[serde_inline_default(false)]
    talk_enabled: bool,
    #[serde_inline_default(false)]
    talk_recording_enabled: bool,
    #[serde_inline_default(false)]
    fulltextsearch_enabled: bool,
    #[serde_inline_default(false)]
    clamav_enabled: bool,
    #[serde_inline_default(false)]
    imaginary_enabled: bool,
}

impl NcAioConfig {
    pub fn create() -> Self {
        let cfg: NcAioConfig = serde_json::from_str("{}")
            .expect("Failed to create ncp config");
        cfg
    }

}

impl Default for NcAioConfig {
    fn default() -> Self {
        NcAioConfig::create()
    }
}

impl CryptoValueProvider<HashMap<String, String>> for NcAioConfig {
    fn get_crypto_value(&self, crypto: &Crypto) -> Result<HashMap<String, String>, NcpError> {
        Ok(HashMap::from([
            self.db_password.kv(crypto)?,
            self.fulltextsearch_pw.kv(crypto)?,
            self.nc_password.kv(crypto)?,
            self.onlyoffice_secret.kv(crypto)?,
            self.recording_secret.kv(crypto)?,
            self.redis_password.kv(crypto)?,
            self.signaling_secret.kv(crypto)?,
            self.talk_internal_secret.kv(crypto)?,
            self.turn_secret.kv(crypto)?
        ]))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationSecrets {
    pub key_derivation_key_nonce: String,
    pub key_derivation_key: String,
    pub key_encryption_key_salt: String,
    pub ncp_version: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcpConfig {
    pub nc_aio: NcAioConfig,
    pub kdk: KeyDerivationSecrets,
    pub ncp_version: String
}

impl NcpConfig {
    pub fn new(ncp_version: &str, crypto: &Crypto) -> Result<Self, NcpError> {
        Ok(Self {
            kdk: crypto.try_into()
                .map_err(|e| format!("Failed to retrieve crypto config: {}", e))?,
            nc_aio: NcAioConfig::create(),
            ncp_version: ncp_version.into()
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), NcpError> {
        serde_json::to_writer(&File::create(path)?, self)
            .map_err(|e| format!("Failed to save configuration: {}", e))?;
        Ok(())
    }
}
