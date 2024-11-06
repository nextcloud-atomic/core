use std::cell::Ref;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use anyhow::{anyhow, bail, Result};
use strum_macros::EnumString;
use std::str::FromStr;
use ring::aead::chacha20_poly1305_openssh::KEY_LEN;
use secrets::SecretVec;
use kvp::KeyValueProvider;
use macros::{secret, KeyValueProvider};
use crate::crypto;
use crate::crypto::{Salt, serde_encode, serde_decode_sized, generate_salt, LockableSecret, create_key_from_pass, Unlockable, secret_to_secret_string};

#[secret]
#[serde_inline_default]
#[derive(Debug, Clone, Serialize, Deserialize, KeyValueProvider)]
pub struct NcAioConfig<'a> {

    #[secret(derived = "AIO_DATABASE_PASSWORD")]
    pub db_password: LockableSecret<'a>,
    #[secret(derived = "AIO_FULLTEXTSEARCH_PASSWORD")]
    pub fulltextsearch_pw: LockableSecret<'a>,
    #[secret(encrypted = "AIO_NEXTCLOUD_PASSWORD")]
    pub nc_password: LockableSecret<'a>,
    #[secret(derived = "AIO_ONLYOFFICE_SECRET")]
    pub onlyoffice_secret: LockableSecret<'a>,
    #[secret(derived = "AIO_RECORDING_SECRET")]
    pub recording_secret: LockableSecret<'a>,
    #[secret(derived = "AIO_REDIS_PASSWORD")]
    pub redis_password: LockableSecret<'a>,
    #[secret(derived = "AIO_SIGNALING_SECRET")]
    pub signaling_secret: LockableSecret<'a>,
    #[secret(derived = "AIO_TALK_INTERNAL_SECRET")]
    pub talk_internal_secret: LockableSecret<'a>,
    #[secret(derived = "AIO_TURN_SECRET")]
    pub turn_secret: LockableSecret<'a>,

    #[serde_inline_default(String::from("nextcloudatomic.local"))]
    pub nc_domain: String,
    #[serde_inline_default(false)]
    pub onlyoffice_enabled: bool,
    #[serde_inline_default(false)]
    pub collabora_enabled: bool,
    #[serde_inline_default(false)]
    pub talk_enabled: bool,
    #[serde_inline_default(false)]
    pub talk_recording_enabled: bool,
    #[serde_inline_default(false)]
    pub fulltextsearch_enabled: bool,
    #[serde_inline_default(false)]
    pub clamav_enabled: bool,
    #[serde_inline_default(false)]
    pub imaginary_enabled: bool,
}

impl<'a> NcAioConfig<'a> {
    pub fn create() -> Self {
        let cfg: NcAioConfig<'a> = serde_json::from_str("{}")
            .expect("Failed to create NC Atomic core");
        cfg
    }
}

impl Default for NcAioConfig<'_> {
    fn default() -> Self {
        NcAioConfig::create()
    }
}

#[secret]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcaConfig<'a> {
    pub nc_aio: NcAioConfig<'a>,
    pub ncatomic_version: String,
    #[secret(encrypted = "NCA_ADMIN_PASSWORD")]
    pub admin_password: LockableSecret<'a>,
    
    #[serde(skip)]
    masterkey: Option<&'a SecretVec<u8>>,
    #[serde(serialize_with = "serde_encode", deserialize_with = "serde_decode_sized")]
    salt: Salt,
}

impl<'a> NcaConfig<'a> {
    pub fn new(ncatomic_version: &str, masterkey: Option<&'a SecretVec<u8>>) -> Result<Self> {
        let salt = generate_salt();
        Ok(Self {
            masterkey,
            salt,
            admin_password: LockableSecret::new_empty_locked(),
            nc_aio: NcAioConfig::create(),
            ncatomic_version: ncatomic_version.into()
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        serde_json::to_writer(&File::create(path)?, self)
            .map_err(|e| anyhow!("Failed to save configuration: {}", e))?;
        Ok(())
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> Result<NcaConfig<'a>> {
        serde_json::from_reader(File::open(path)?)
            .map_err(|e| anyhow!("Failed to load configuration: {}", e))
    }

    pub fn is_locked(&self) -> bool {
        self.masterkey.is_some()
    }

    pub fn unlock(&mut self, masterkey: &'a SecretVec<u8>) -> Result<()> {
        // let masterkey = crypto::create_key_from_pass(self.salt, password);
        self.unlock_secrets(masterkey)?;
        self.masterkey = Some(masterkey);


        Ok(())
    }


    fn unlock_secrets(&mut self, key: &'a SecretVec<u8>) -> Result<()> {
        self.admin_password = self.admin_password.unlock(key, self.salt);
        self.nc_aio.unlock(key, self.salt)
            .map_err(|e| anyhow!("Failed to unlock: {}", e))
    }

    pub fn get_masterkey(&self, password: SecretVec<u8>) -> SecretVec<u8> {
        create_key_from_pass(self.salt, password)
    }
    
    pub fn get_salt(&self) -> Salt {
        self.salt
    }
}


