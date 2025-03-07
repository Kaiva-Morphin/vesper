use axum::http::HeaderMap;

pub fn get_user_agent(headers: &HeaderMap) -> String {
    headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("unknown").to_string()
}

pub fn get_user_ip(headers: &HeaderMap) -> String {
    headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()).unwrap_or("unknown").to_string()
}
