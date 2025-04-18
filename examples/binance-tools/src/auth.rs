use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[allow(dead_code)]
type HmacSha256 = Hmac<Sha256>;

#[allow(unused)]
/// get hmac signature(hex)
pub fn get_hmac_signature(secret_key: &str, data: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())?;
    mac.update(data.as_bytes());
    let result = mac.finalize().into_bytes();

    Ok(hex::encode(result))
}
