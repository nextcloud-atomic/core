use std::num::NonZeroU32;
use ring::{aead, hkdf, pbkdf2};
use ring::aead::{AES_256_GCM, NONCE_LEN};
use ring::hkdf::HKDF_SHA256;
use ring::pbkdf2::PBKDF2_HMAC_SHA512;
use ring::rand::SecureRandom;

static KEK_CIPHER: &aead::Algorithm = &AES_256_GCM;
const KEK_LENGTH: usize = 256 / 8;
// const KEY_LENGTH: usize = 512 / 8;
const SALT_LENGTH: usize = 16;
const B32_ENCODING_ALPHABET: base32::Alphabet = base32::Alphabet::Rfc4648 {padding: true};
pub type Salt = [u8; SALT_LENGTH];
pub type Nonce = [u8; NONCE_LEN];
pub type AesKey = [u8; KEK_LENGTH];

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

pub fn generate_salt() -> Salt {
    let rng = ring::rand::SystemRandom::new();
    let mut buf = [0; SALT_LENGTH];
    rng.fill(&mut buf).expect("Unexpectedly failed to fill salt buffer");
    buf
}

pub fn generate_nonce() -> Nonce {
    let rng = ring::rand::SystemRandom::new();
    let mut buf = [0; NONCE_LEN];
    rng.fill(&mut buf).expect("Unexpectedly failed to fill nonce");
    buf
}

pub fn create_key_from_pass(salt: &Salt, pass: String) -> AesKey {
    let mut buf = [0u8; KEK_LENGTH];
    pbkdf2::derive(PBKDF2_HMAC_SHA512, NonZeroU32::new(100_000).unwrap(), salt, pass.as_bytes(), &mut buf);
    buf
}

pub fn derive_key(primary_key: &AesKey, salt: &Salt, secret_id: String) -> Result<AesKey, String> {
    let hkdf_salt = hkdf::Salt::new(HKDF_SHA256, salt);
    let mut okm: Vec<u8> = match hkdf_salt
        .extract(primary_key)
        .expand(&[secret_id.as_bytes()], HkdfMy(KEK_LENGTH)) {
        Ok(my_okm) => {
            let HkdfMy(okm) = my_okm.into();
            okm
        },
        Err(_) => {
            return Err("Failed to derive okm".to_string());
        }
    };
    assert_eq!(KEK_LENGTH, okm.len());
    let mut result = [0u8; KEK_LENGTH];
    result.copy_from_slice(&okm);
    Ok(result)
}


pub fn b32_encode(data: &[u8]) -> String {
    base32::encode(B32_ENCODING_ALPHABET, data)
}
pub fn b32_decode(data: &str) -> Result<Vec<u8>, String> {
    // let t = [0u8, 1u8, 2u8];
    // let v = Vec::from(t);
    base32::decode(B32_ENCODING_ALPHABET, data)
        .ok_or("Failed to decode string".to_string())
}