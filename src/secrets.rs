use std::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};
use crate::crypto;
use crate::crypto::{Crypto, CryptoValueProvider};
use crate::error::NcpError;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DerivedSecret {
    key: String,
}

impl DerivedSecret {
    pub fn new(key: &str) -> Self {
        Self{
            key: key.to_string(),
        }
    }
    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn kv(&self, crypto: &Crypto) -> Result<(String, String), NcpError> {
        Ok((self.get_key().to_string(), self.get_crypto_value(crypto)?))
    }
}

impl CryptoValueProvider<String> for DerivedSecret {
    fn get_crypto_value(&self, crypto: &Crypto) -> Result<String, NcpError> {
        Ok(crypto::encode(&crypto.derive_secret(&self.key)?))
    }
}

impl Debug for DerivedSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DerivedSecret(key: {})", self.key)
    }
}

impl From<&str> for DerivedSecret {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

