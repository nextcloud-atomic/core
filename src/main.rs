use hkdf::Hkdf;
use sha2::Sha256;
use bcrypt::{BcryptError, DEFAULT_COST, hash_with_salt, Version};
use hex_literal::hex;
use rand::random;


const INFO: [u8; 10] = hex!("f0f1f2f3f4f5f6f7f8f9");

fn create_master_key(salt: &[u8; 16], pass: &str) -> Result<String, BcryptError> {
    let result = hash_with_salt(pass, DEFAULT_COST, salt)?;
    Ok(result.format_for_version(Version::TwoB))
}

fn generate_salt() -> [u8; 16] {
    let ar: [u8; 16] = random();
    return ar;
}
fn derive_key(salt: &[u8], mk: &[u8]) -> Result<[u8; 42], hkdf::InvalidLength> {
    let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &mk);
    let mut okm = [0u8; 42];
    hk.expand(&INFO, &mut okm)?;
    Ok(okm)
}

fn main() {

    let master_salt = generate_salt();
    let master_key = create_master_key(&master_salt, "testpw")
        .expect("Failed to create master key from password");
    let derived_salt = generate_salt();
    let derived = derive_key(&derived_salt, master_key.as_bytes());
    let mk_bytes = master_key.as_bytes();

    println!("master key: {mk_bytes:?}\nmaster key salt: {master_salt:?}\nderived_salt: {derived_salt:?}\nderived: {derived:?}")
}
