use axum::http::HeaderMap;

pub const UNKNOWN_FINGERPRINT : &'static str = "unknown_fingerprint";
pub const UNKNOWN_USER_AGENT : &'static str = "unknown_user_agent";
pub const UNKNOWN_IP : &'static str = "unknown_ip";

pub fn get_user_agent(headers: &HeaderMap) -> String {
    headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or(UNKNOWN_USER_AGENT).to_string()
}

pub fn get_user_ip(headers: &HeaderMap) -> String {
    headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()).unwrap_or(UNKNOWN_IP).to_string()
}

pub fn get_user_fingerprint(headers: &HeaderMap) -> String {
    headers.get("Fingerprint").and_then(|v| v.to_str().ok()).unwrap_or(UNKNOWN_FINGERPRINT).to_string()
}
