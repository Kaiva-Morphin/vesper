use base64::engine::general_purpose;
use base64::Engine;
use rand::RngCore;

pub fn generate_secure_token(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}