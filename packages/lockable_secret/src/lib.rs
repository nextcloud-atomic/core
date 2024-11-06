#![feature(slice_pattern)]

use core::slice::SlicePattern;
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;
use hex_literal::hex;
use ring::{aead, digest, hkdf, pbkdf2};
use ring::aead::{Aad, AES_256_GCM, LessSafeKey, Nonce, NONCE_LEN, UnboundKey};
use ring::digest::{digest};
use ring::hkdf::{HKDF_SHA256};
use ring::pbkdf2::PBKDF2_HMAC_SHA512;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use anyhow::{anyhow, bail, Result};
use secrets::{SecretVec};
use serde::de::{Error, Visitor};
// reference: https://github.com/neosmart/securestore-rs/blob/master/securestore/src/shared.rs

const B32_ENCODING_ALPHABET: base32::Alphabet = base32::Alphabet::Rfc4648 {padding: true};
const INFO: [u8; 10] = hex!("f0f1f2f3f4f5f6f7f8f9");
static KEK_CIPHER: &aead::Algorithm = &AES_256_GCM;

const KEY_LENGTH: usize = 512 / 8;
const SALT_LENGTH: usize = 16;

pub type Salt = [u8; SALT_LENGTH];

pub fn encode(data: &[u8]) -> String {
    base32::encode(B32_ENCODING_ALPHABET, data)
}
pub fn decode(data: &str) -> Result<Vec<u8>> {
    // let t = [0u8, 1u8, 2u8];
    // let v = Vec::from(t);
    base32::decode(B32_ENCODING_ALPHABET, data)
        .ok_or(anyhow::Error::msg("Failed to decode string"))
}

struct ByteStringVisitor;
impl <'de> Visitor<'de> for ByteStringVisitor {
    type Value = Vec<u8>;
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("string expected")
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(decode(v).map_err(E::custom)?)
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(decode(v).map_err(E::custom)?)
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(decode(v.as_str()).map_err(E::custom)?)
    }
}
pub fn serde_decode<'de, D: Deserializer<'de>, T: From<Vec<u8>>>(d: D) -> Result<T, D::Error> {
    let data = d.deserialize_str(ByteStringVisitor)?;
    Ok(T::from(data))
}

pub fn serde_decode_sized<'de, D: Deserializer<'de>, const L: usize>(d: D) -> Result<[u8; L], D::Error> {
    let data = d.deserialize_str(ByteStringVisitor)?;
    let mut result = [0u8; L];
    result.copy_from_slice(data.as_slice());
    Ok(result)
}

pub fn serde_encode<S: Serializer, T: AsRef<[u8]>>(data: T, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(encode(data.as_ref()).as_str())
}

// pub fn serde_decode_vec<'de, D: Deserializer<'de>>(data: &str, d: D) -> Result<Vec<u8>, D::Error> {
//     serde_decode(data, d).map(|b| b.to_vec())
// }

pub struct EncryptionResult {
    pub data: Vec<u8>,
    pub nonce: [u8; NONCE_LEN],
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LockableSecret<'a> {
    #[serde(rename = "locked")]
    Locked(LockedSecret),
    #[serde(skip_deserializing, rename = "locked", serialize_with = "serialize_unlocked")]
    Unlocked(UnlockedSecret<'a>)
}

fn serialize_unlocked<S: Serializer>(unlocked: &UnlockedSecret, s: S) -> Result<S::Ok, S::Error> {
    unlocked.secret_data.serialize(s)
}

impl<'a> LockableSecret<'a> {

    pub fn is_locked(&self) -> bool {
        match self {
            LockableSecret::Locked(_) => true,
            LockableSecret::Unlocked(_) => false
        }
    }
    pub fn unlock(&self, masterkey: &'a SecretVec<u8>, salt: Salt) -> LockableSecret<'a> {
        match self {
            LockableSecret::Unlocked(_) => self.clone(),
            LockableSecret::Locked(locked) => {
                let unlocked = UnlockedSecret{
                    secret_data: locked.clone(),
                    key: masterkey,
                    salt,
                };
                LockableSecret::Unlocked(unlocked)
            }
        }
    }

    pub fn lock(&self) -> LockableSecret<'a> {

        let locked = match self {
            LockableSecret::Locked(secret) => secret,
            LockableSecret::Unlocked(unlocked) => &unlocked.secret_data
        }.clone();
        LockableSecret::Locked(locked)
    }

    pub fn new_encrypted(key: &'a SecretVec<u8>, salt: Salt, version: String, plaintext: SecretVec<u8>) -> Result<LockableSecret<'a>> {
        let encrypted = encrypt_vec(key, plaintext, salt, format!("version::{version}"))?;
        Ok(LockableSecret::Unlocked(UnlockedSecret{
            key,
            salt,
            secret_data: LockedSecret::ENCRYPTED(EncryptedSecret {
                source_version: version,
                data: encrypted.data,
                nonce: encrypted.nonce
            })
        }))
    }

    pub fn new_derived(key: &'a SecretVec<u8>, secret_id: String, salt: Salt) -> Result<LockableSecret<'a>> {
        Ok(LockableSecret::Unlocked(UnlockedSecret {
            key,
            salt,
            secret_data: LockedSecret::DERIVED(DerivedSecret {
                secret_id
            })
        }))
    }

    pub fn new_derived_locked<S: Into<String>>(secret_id: S) -> LockableSecret<'a> {
        LockableSecret::Locked(LockedSecret::DERIVED(DerivedSecret{
            secret_id: secret_id.into(),
        }))
    }

    pub fn new_empty_locked() -> LockableSecret<'a> {
        LockableSecret::Locked(LockedSecret::EMPTY)
    }

    pub fn secret_value(&self) -> Result<SecretVec<u8>> {
        match self {
            LockableSecret::Locked(sec) => {
                match sec {
                    LockedSecret::EMPTY => Ok(SecretVec::zero(0)),
                    _ => bail!("secret is locked")
                }
            }
            LockableSecret::Unlocked(unlocked) => {
                Ok(unlocked.clone().secret_value()?)
            }
        }
    }


    pub fn encode_secret_value(&self) -> Result<String> {
        let val = encode(self.secret_value()?.borrow().iter().as_slice());
        Ok(val)
    }
}

pub fn secret_to_secret_string(sec: &LockableSecret) -> String {
    match sec.encode_secret_value() {
        Err(e) => panic!("Failed to get secret value: {:?}", e),
        Ok(s) => s,
    }
}

// impl Serialize for LockableSecret<'_> {
//     fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         if let LockableSecret::Locked(_) = self {
//             return self.lock().serialize(serializer)
//         }
//
//         self.serialize(serializer)
//     }
// }

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum LockedSecret {
    ENCRYPTED(EncryptedSecret),
    DERIVED(DerivedSecret),
    EMPTY
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EncryptedSecret {
    #[serde(serialize_with = "serde_encode", deserialize_with = "serde_decode")]
    data: Vec<u8>,
    #[serde(serialize_with = "serde_encode", deserialize_with = "serde_decode_sized")]
    nonce: [u8; NONCE_LEN],
    source_version: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct DerivedSecret {
    secret_id: String,
}

impl<'a> LockedSecret {
    pub fn unlock(&self, key: &'a SecretVec<u8>, salt: Salt) -> UnlockedSecret<'a> {
        UnlockedSecret{
            key,
            salt,
            secret_data: self.clone()
        }
    }

}

#[derive(Clone, PartialEq, Debug)]
pub struct UnlockedSecret<'a> {
    secret_data: LockedSecret,
    key: &'a SecretVec<u8>,
    salt: Salt,
}

impl UnlockedSecret<'_> {

    pub fn secret_value(self) -> Result<SecretVec<u8>> {
        match self.secret_data {
            LockedSecret::DERIVED(secret) => {
                Ok(derive_secret(self.key, secret.secret_id.clone())?)
            },
            LockedSecret::ENCRYPTED(secret) => {
                Ok(decrypt_vec(self.key, Vec::from(secret.data.clone()), secret.nonce, self.salt, format!("version::{}", secret.source_version))?)
            },
            LockedSecret::EMPTY => {
                Ok(SecretVec::<u8>::zero(0))
            }
        }
    }
}


pub fn generate_salt() -> Salt {
    let rng = ring::rand::SystemRandom::new();
    let mut buf = [0; 16];
    rng.fill(&mut buf).expect("Failed to fill buffer for salt");
    buf
}
fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce_bytes = vec![0u8; NONCE_LEN];
    let rng = SystemRandom::new();
    rng.fill(&mut nonce_bytes).expect("failed to generate random nonce");
    nonce_bytes.try_into().expect("failed to convert nonce")
}

fn encrypt_vec(key: &SecretVec<u8>, data: SecretVec<u8>, salt: Salt, associated_data: String) -> Result<EncryptionResult> {
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|e| anyhow!("Failed to generate nonce: {e:?}"))?;
    let encryption_key = LessSafeKey::new(
        UnboundKey::new(&KEK_CIPHER, &*key.borrow())
            .map_err(|e| anyhow!("Failed to create unbound key: {e:?}"))?);

    let mut ciphertext = vec![0u8; data.len()];
    ciphertext.copy_from_slice(data.borrow().as_slice());
    encryption_key.seal_in_place_append_tag(
        nonce,
        Aad::from([associated_data.as_bytes(), salt.as_slice()].concat()),
        &mut ciphertext
    )
        .map_err(|e| anyhow!("Failed to encrypt: {e}"))?;
    Ok(EncryptionResult{
        data: ciphertext,
        nonce: nonce_bytes,
    })
}

fn decrypt_vec(key: &SecretVec<u8>, data: Vec<u8>, nonce_bytes: [u8; NONCE_LEN], salt: Salt, associated_data: String) -> Result<SecretVec<u8>> {
    let enc = decrypt_vec_unsafe(key, data, nonce_bytes, salt, associated_data)?;
    Ok(SecretVec::<u8>::new(enc.len(), |s| {
        s.copy_from_slice(enc.as_slice());
    }))
}

fn decrypt_vec_unsafe(key: &SecretVec<u8>, data: Vec<u8>, nonce_bytes: [u8; NONCE_LEN], salt: Salt, associated_data: String) -> Result<Vec<u8>> {

    let decryption_key = LessSafeKey::new(
        UnboundKey::new(&KEK_CIPHER, &*key.borrow())
            .map_err(|e| anyhow!("{e:?}"))?);
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|e| anyhow!("{e:?}"))?;
    let mut in_out = data;
    let decrypted = decryption_key.open_in_place(
        nonce,
        Aad::from([associated_data.as_bytes(), salt.as_slice()].concat()),
        &mut in_out,
    ).map_err(|e| anyhow!("Failed to decrypt: {e}"))?;
    Ok(decrypted.to_vec())
}

fn derive_secret(key: &SecretVec<u8>, secret_id: String) -> Result<SecretVec<u8>> {
    let sha256 = digest(&digest::SHA256, secret_id.as_bytes());
    let hdkf_salt = hkdf::Salt::new(HKDF_SHA256, sha256.as_ref());
    let mut okm: Vec<u8> = match hdkf_salt.extract(&*key.borrow()).expand(&[&INFO], HkdfMy(42)) {
        Ok(my_okm) => {
            let HkdfMy(okm) = my_okm.into();
            okm
        }
        Err(_) => bail!("Failed to derive okm")
    };
    assert_eq!(42, okm.len(), "derived secret must be of length 42, was {}!", okm.len());
    let ciphertext = SecretVec::<u8>::new(42, |s| {
        s.copy_from_slice(&mut okm);
    });
    Ok(ciphertext)
}

pub fn create_key_from_pass(salt: [u8; 16], pass: SecretVec<u8>) -> SecretVec<u8> {
    let length = KEK_CIPHER.key_len();
    SecretVec::<u8>::new(length, |mut s| {
        pbkdf2::derive(PBKDF2_HMAC_SHA512, NonZeroU32::new(100_000).unwrap(), &salt, pass.borrow().as_slice(), &mut s);
    })
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

pub trait Unlockable<'a> {
    fn unlock(&mut self, key: &'a SecretVec<u8>, salt: Salt) -> std::result::Result<(), String>;
}


#[cfg(test)]
mod tests {
    use core::slice::SlicePattern;
    use super::*;
    use std::borrow::Borrow;
    use std::str;

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

    #[test]
    fn test_de_serialization_encryption() {
        let masterkey = SecretVec::<u8>::random(KEK_CIPHER.key_len());
        let salt = generate_salt();

        let plaintext = SecretVec::new(6, |mut s| {
            let mut v = "abcdef".as_bytes();
            s.copy_from_slice(&mut v)
        });

        let sec = LockableSecret::new_encrypted(&masterkey, salt, "0.0.1".to_string(), plaintext.clone())
            .expect("Failed to create encrypted secret");
        let serialized = serde_json::to_string(&sec).expect("failed to serialize to json");
        print!("{serialized}\n");
        let deserialized: LockableSecret = serde_json::from_str(&serialized).expect("Failed to deserialize lockale secret from json");

        assert!(deserialized.is_locked(), "secret not locked after deserialization!");

        let unlocked = deserialized.unlock(&masterkey, salt);
        assert!(!unlocked.is_locked(), "secret locked after unlocking!");
        let secret_value = unlocked.secret_value().expect("failed to retriee secret value");
        assert_eq!(plaintext.borrow(), secret_value.borrow());
    }

    #[test]
    fn test_de_serialization_derivation() {
        let masterkey = SecretVec::<u8>::random(KEK_CIPHER.key_len());
        let salt = generate_salt();

        let secret_id = "TEST_1".to_string();

        let sec = LockableSecret::new_derived(&masterkey, secret_id, salt)
            .expect("Failed to create derived secret");
        let serialized = serde_json::to_string(&sec).expect("failed to serialize to json");
        print!("{serialized}\n");
        let deserialized: LockableSecret = serde_json::from_str(&serialized).expect("Failed to deserialize lockale secret from json");

        assert!(deserialized.is_locked(), "secret not locked after deserialization!");

        let unlocked = deserialized.unlock(&masterkey, salt);
        assert!(!unlocked.is_locked(), "secret locked after unlocking!");
        let secret_value = unlocked.secret_value().expect("failed to retriee secret value");
        assert_eq!(sec.secret_value().unwrap().borrow(), secret_value.borrow());
    }

    #[test]
    fn test_de_encryption() {
        let plaintext = SecretVec::new(6, |mut s| {
            let mut v = "abcdef".as_bytes();
            s.copy_from_slice(&mut v)
        });

        println!("plaintext length: {} | {}", plaintext.len(), plaintext.borrow().len());
        let salt = generate_salt();

        let masterkey = SecretVec::<u8>::random(KEK_CIPHER.key_len());

        let encrypted = encrypt_vec(&masterkey, plaintext, salt, "test".to_string())
            .expect("Failed to encrypt data");
        println!("cipher text length: {}", encrypted.data.len());
        let decrypted = decrypt_vec_unsafe(&masterkey, encrypted.data, encrypted.nonce, salt, "test".to_string())
            .expect("Failed to decrypt data");

        assert_eq!("abcdef", decrypted.iter().map(|&c| char::from(c)).collect::<String>());

    }

    #[test]
    fn unique_nonce() {
        let masterkey = SecretVec::<u8>::random(KEK_CIPHER.key_len());
        let salt = generate_salt();

        let plaintext = SecretVec::new(6, |mut s| {
            let mut v = "abcdef".as_bytes();
            s.copy_from_slice(&mut v)
        });
        let encrypted_1 = encrypt_vec(&masterkey, plaintext, salt, "test".to_string())
            .expect("Failed to encrypt data");
        let nonce_1 = encrypted_1.nonce;
        let decrypted = decrypt_vec(&masterkey, encrypted_1.data, encrypted_1.nonce, salt, "test".to_string())
            .expect("Failed to decrypt data");
        let encrypted_2 = encrypt_vec(&masterkey, decrypted, salt, "test".to_string())
            .expect("Failed to encrypt data");

        assert_ne!(encrypted_1.nonce, encrypted_2.nonce);
    }

    #[test]
    fn deterministic_secrets() {
        let masterkey = SecretVec::<u8>::random(KEK_CIPHER.key_len());
        let salt = generate_salt();

        let derived_1 = LockableSecret::new_derived(&masterkey, "TEST_1".to_string(), salt)
            .expect("Failed to derive secret");

        let derived_2 = LockableSecret::new_derived(&masterkey, "TEST_1".to_string(), salt)
            .expect("Failed to derive secret");

        let derived_3 = LockableSecret::new_derived(&masterkey, "TEST_2".to_string(), salt)
            .expect("Failed to derive secret");

        assert_eq!(derived_1.secret_value().unwrap(), derived_2.secret_value().unwrap());
        assert_ne!(derived_1.secret_value().unwrap(), derived_3.secret_value().unwrap());
    }

}
