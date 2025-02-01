use axum::http::HeaderMap;


















pub fn get_user_agent(headers: &HeaderMap) -> String {
    headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("NOT_PROVIDED").to_string()
}


