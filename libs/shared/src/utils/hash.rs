use sha2::Digest;

use crate::utils::header::UNKNOWN_FINGERPRINT;

pub fn hash_fingerprint(fp: &String) -> String {
    if fp == UNKNOWN_FINGERPRINT {return UNKNOWN_FINGERPRINT.to_string()}
    format!("{:x}", sha2::Sha256::digest(fp.as_bytes()))
}