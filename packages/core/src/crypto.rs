use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;
use hex_literal::hex;
use ring::{aead, digest, hkdf, pbkdf2};
use ring::aead::{Aad, AES_256_GCM, LessSafeKey, Nonce, NONCE_LEN, UnboundKey};
use ring::digest::{digest, SHA512_OUTPUT_LEN};
use ring::hkdf::{HKDF_SHA256};
use ring::pbkdf2::PBKDF2_HMAC_SHA512;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::config::{KeyDerivationSecrets, NcaConfig};
use anyhow::{anyhow, bail, Result};

const B32_ENCODING_ALPHABET: base32::Alphabet = base32::Alphabet::Rfc4648 {padding: true};
const INFO: [u8; 10] = hex!("f0f1f2f3f4f5f6f7f8f9");
static KEK_CIPHER: &aead::Algorithm = &AES_256_GCM;

pub fn encode(data: &[u8]) -> String {
    base32::encode(B32_ENCODING_ALPHABET, data)
}
pub fn decode(data: &str) -> Result<Vec<u8>> {
    base32::decode(B32_ENCODING_ALPHABET, data)
        .ok_or(anyhow::Error::msg("Failed to decode string"))
}

#[derive(Clone, Deserialize)]
pub struct Crypto {
    locked: bool,
    ncatomic_version: String,
    kek: Option<Vec<u8>>,
    kek_salt: [u8; 16],
    kdk: Vec<u8>,
    kdk_nonce: [u8; NONCE_LEN],
}
impl Crypto {

    pub fn new(ncatomic_version: &str, pass: &str) -> Result<Self> {
        let kek_salt = Self::generate_salt()?;

        let mut kdk = vec![0u8; SHA512_OUTPUT_LEN];

        let rng = SystemRandom::new();
        rng.fill(&mut kdk)?;
        let nonce_bytes = Self::generate_nonce()?;

        let mut crypto = Self::load_unsafe(&encode(&kek_salt), &encode(&kdk), &encode(&nonce_bytes), ncatomic_version, false)?;
        let kek_secret = Self::create_key_from_pass(kek_salt, pass, KEK_CIPHER.key_len());
        crypto.lock_unsafe(kek_secret)?;
        crypto.unlock(pass)?;
        Ok(crypto)
    }

    fn generate_nonce() -> Result<[u8; 12]> {
        let mut nonce_bytes = vec![0u8; NONCE_LEN];
        let rng = SystemRandom::new();
        rng.fill(&mut nonce_bytes).map_err(|_| anyhow!("failed to generate random nonce"))?;
        Ok(nonce_bytes.try_into().map_err(|_| anyhow!("failed to convert nonce"))?)
    }

    fn load_unsafe(kek_salt_str: &str, kdk_data_str: &str, kdk_nonce_str: &str, ncatomic_version: &str, locked: bool) -> Result<Self> {
        let kek_salt = decode(kek_salt_str)?;
        let kdk_data = decode(kdk_data_str)?;
        let kdk_nonce_bytes = decode(kdk_nonce_str)?;

        Ok(Crypto {
            ncatomic_version: ncatomic_version.to_string(),
            locked,
            kek: None,
            kek_salt: kek_salt.try_into().map_err(|_| anyhow!("Failed to convert salt"))?,
            kdk: kdk_data,
            kdk_nonce: kdk_nonce_bytes.try_into().map_err(|_| anyhow!("Failed to convert nonce"))?,
        })
    }

    pub fn load(kek_salt_str: &str, kdk_data_str: &str, kdk_nonce_str: &str, ncatomic_version: &str) -> Result<Self> {
        Self::load_unsafe(kek_salt_str, kdk_data_str, kdk_nonce_str, ncatomic_version, true)
    }

    pub fn is_locked(&self) -> bool {
        return self.locked;
    }
    pub fn unlock(&mut self, pass: &str) -> Result<()> {
        let kek_secret = Self::create_key_from_pass(self.kek_salt, pass, KEK_CIPHER.key_len());
        let kek = LessSafeKey::new(UnboundKey::new(&KEK_CIPHER, &kek_secret)
            .map_err(|e| anyhow!("failed to init kek: {e}"))?);

        let nonce = Nonce::try_assume_unique_for_key(&self.kdk_nonce)
            .map_err(|e| anyhow!("failed to init nonce: {e}"))?;
        let mut in_out = self.kdk.clone();
        let decrypted = kek.open_in_place(nonce,
                                          Aad::from(format!("ncatomic::{}", self.ncatomic_version).as_bytes()),
                                          &mut in_out).map_err(|e| anyhow!("failed to open kdk: {e}"))?;
        self.kdk = decrypted.into();

        self.kek = Some(kek_secret.try_into().map_err(|e| anyhow!("failed to store kek_secret: {e}"))?);
        self.kdk_nonce = Self::generate_nonce()?;
        self.locked = false;
        Ok(())
    }

    pub fn lock(&mut self) -> Result<()> {
        let kek_bytes = self.kek.clone().ok_or(anyhow!("key encryption key is not set"))?;
        self.lock_unsafe(kek_bytes)
    }
    fn lock_unsafe(&mut self, kek_bytes: Vec<u8>) -> Result<()> {
        let kek = LessSafeKey::new(UnboundKey::new(&KEK_CIPHER, &*kek_bytes)?);

        let mut nonce_bytes = vec![0u8; NONCE_LEN];
        let rng = SystemRandom::new();
        rng.fill(&mut nonce_bytes)?;
        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)?;

        kek.seal_in_place_append_tag(nonce,
                                     Aad::from(format!("ncatomic::{}", self.ncatomic_version).as_bytes()),
                                     &mut self.kdk).map_err(|e| anyhow!("Failed to encrypt: {e}"))?;
        self.locked = true;
        self.kdk_nonce = nonce_bytes.try_into()
            .map_err(|_| anyhow!("Failed to parse nonce"))?;
        self.kek = None;
        Ok(())
    }

    fn create_key_from_pass(salt: [u8; 16], pass: &str, length: usize) -> Vec<u8> {
        let mut derived = vec![0u8; length];
        pbkdf2::derive(PBKDF2_HMAC_SHA512, NonZeroU32::new(100_000).unwrap(), &salt, pass.as_bytes(), &mut derived);
        derived
    }

    fn generate_salt() -> Result<[u8; 16]> {
        let rng = ring::rand::SystemRandom::new();
        let mut buf = [0; 16];
        match rng.fill(&mut buf) {
            Err(_) => bail!("Failed to fill buffer for salt"),
            Ok(_) => Ok(buf)
        }
    }

    pub fn derive_secret(&self, secret_key: &str) -> Result<Vec<u8>> {
        if self.locked {
            bail!("crypto is not unlocked")
        }
        let sha256 = digest(&digest::SHA256, secret_key.as_bytes());
        let hdkf_salt = hkdf::Salt::new(HKDF_SHA256, sha256.as_ref());
        match hdkf_salt.extract(&self.kdk).expand(&[&INFO], HkdfMy(42)) {
            Ok(my_okm) => {
                let HkdfMy(okm) = my_okm.into();
                Ok(okm)
            }
            Err(_) => bail!("Failed to derive okm")
        }
    }

}

impl TryInto<KeyDerivationSecrets> for &Crypto {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<KeyDerivationSecrets, Self::Error> {
        let locked_crypto = match self.locked {
            true => self.clone(),
            false => {
                let mut crypto = self.clone();
                crypto.lock()?;
                crypto
            }
        };
        Ok(KeyDerivationSecrets {
            key_derivation_key_nonce: encode(&locked_crypto.kdk_nonce),
            key_encryption_key_salt: encode(&locked_crypto.kek_salt),
            key_derivation_key: encode(&locked_crypto.kdk),
            ncatomic_version: locked_crypto.ncatomic_version.to_string()
        })
    }
}

impl TryFrom<KeyDerivationSecrets> for Crypto {
    type Error = anyhow::Error;
    fn try_from(config: KeyDerivationSecrets) -> Result<Self, Self::Error> {
        Crypto::load(config.key_encryption_key_salt.as_str(),
                     config.key_derivation_key.as_str(),
                     config.key_derivation_key_nonce.as_str(),
                     config.ncatomic_version.as_str())
    }
}

impl Serialize for Crypto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let copy = match &self.locked {
            true => self.clone(),
            false => {
                let mut c = self.clone();
                // TODO: Error handling
                c.lock().unwrap();
                c
            }
        };

        let mut json = serializer.serialize_struct("crypto", 5)?;
        json.serialize_field("ncatomic_version", &copy.ncatomic_version)?;
        json.serialize_field("locked", &copy.locked)?;
        let none: Option<Vec<u8>> = None;
        json.serialize_field("kek", &none)?;
        json.serialize_field("kek_salt", &encode(&copy.kek_salt))?;
        json.serialize_field("kdk", &encode(&copy.kdk))?;
        json.serialize_field("kdk_nonce", &encode(&copy.kdk_nonce))?;
        json.end()

    }
}

impl Debug for Crypto {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Crypto()")
    }
}

impl From<NcaConfig> for Crypto {
    fn from(cfg: NcaConfig) -> Self {
        Self::load(cfg.kdk.key_encryption_key_salt.as_str(), cfg.kdk.key_derivation_key.as_str(),
                   cfg.kdk.key_derivation_key_nonce.as_str(), cfg.ncatomic_version.as_str())
            .expect("Failed to load crypto from configuration")

    }
}


#[derive(Debug, PartialEq)]
struct HkdfMy<T: core::fmt::Debug + PartialEq>(T);

impl hkdf::KeyType for HkdfMy<usize> {
    fn len(&self) -> usize {
        self.0
    }
}

impl From<hkdf::Okm<'_, HkdfMy<usize>>> for HkdfMy<Vec<u8>> {
    fn from(okm: hkdf::Okm<HkdfMy<usize>>) -> Self {
        let mut r = vec![0u8; okm.len().0];
        okm.fill(&mut r).unwrap();
        Self(r)
    }
}
pub trait CryptoUser<'a> {
    fn set_crypto(&'a mut self, crypto: Option<&'a Crypto>);

}

pub trait CryptoValueProvider<T> {
    fn get_crypto_value(&self, crypto: &Crypto) -> Result<T>;
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::str;

    #[test]
    fn create_crypto() {
        let crypto = Crypto::new("0.0.0", "test")
            .expect("crypto could not be initialized");
        assert_eq!(crypto.locked, false);
        let derived = crypto.derive_secret("abcdef").expect("failed to derive secret");
        println!("{}", encode(&derived));
    }

    #[test]
    fn unique_nonce() {
        let pass = "test";
        let mut crypto = Crypto::new("0.0.0", pass)
            .expect("crypto could not be intialized");
        let first_nonce = crypto.kdk_nonce;
        crypto.lock().expect("crypto could not be locked");
        crypto.unlock(pass).expect("crypto could not be unlocked");
        assert_ne!(first_nonce, crypto.kdk_nonce, "nonce was not regenerated after locking/unlocking");
    }

    #[test]
    fn deterministic_secrets() {
        let pass = "test";
        let secret_key = "abcdef";
        let mut crypto = Crypto::new("0.0.0", pass)
            .expect("crypto could not be initialized");
        println!("kdk: {}, nonce: {}", encode(&crypto.kdk), encode(&crypto.kdk_nonce));
        let secret = crypto.derive_secret(secret_key)
            .expect("secret could not be derived");
        crypto.lock().expect("crypto could not be locked");
        crypto.unlock(pass).expect("crypto could not be unlocked");
        println!("kdk: {}, nonce: {}", encode(&crypto.kdk), encode(&crypto.kdk_nonce));
        assert_eq!(crypto.derive_secret(secret_key).expect("could not derive secret"), secret);
    }

    #[test]
    fn de_serialization() {
        let pass = "test";
        let secret_key = "abcdef";
        let crypto = Crypto::new("0.0.0", pass)
            .expect("could not initialize crypto");
        let secret1 = crypto.derive_secret(secret_key)
            .expect("could not derive secret");
        let serialized: KeyDerivationSecrets = crypto.borrow().try_into()
            .expect("could not serialize crypto");
        let mut crypto2: Crypto = serialized.try_into()
            .expect("could not deserialize crypto");
        crypto2.unlock(pass).expect("could not unlock crypto");
        let secret2 = crypto2.derive_secret(secret_key)
            .expect("could not derive secret");

        assert_eq!(secret1, secret2);

    }

    #[test]
    fn test_en_decode() {
        let text = b"some-test-value";
        let encoded = encode(text);
        let decoded = decode(&encoded).expect("failed to decode");
        let decoded_str = str::from_utf8(decoded.as_slice())
            .expect("failed to convert decoded data to utf8 string");
        println!("{decoded_str}");
        assert_eq!(text, decoded.as_slice())
    }

}
