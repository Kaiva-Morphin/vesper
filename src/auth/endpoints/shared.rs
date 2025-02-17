use axum::{http::HeaderMap, Json};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{shared::{settings::ACCESS_TOKEN_LIFETIME, structs::tokens::{cookies::TokenCookie, tokens::{TokenEncoder, AccessTokenPayload, AccessTokenResponse, RefreshTokenRecord}}}, AppState};









pub async fn process_tokens(
    jar: CookieJar,
    state: &AppState,
    user: Uuid,
    fingerprint: String,
    user_agent: String,
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
let rtid = Uuid::new_v4();
    
    let token_record = RefreshTokenRecord{
        rtid,
        user,
        fingerprint,
        user_agent,
        ip: "Undefined".to_string() // TODO! can be provided after nginx????  println!("\n\n\n{:?}", headers.get("x-forwarded-for"));  println!("\n\n\n{:?}", headers.get("x-real-ip"));
    };
    let now = Utc::now().timestamp();
    
    let access_payload = AccessTokenPayload{
        user,
        created: now,
        lifetime: ACCESS_TOKEN_LIFETIME
    };

    let access_token = TokenEncoder::encode_access(access_payload)?;
    state.tokens.set_refresh(token_record)?;

    Ok((
        jar.put_rtid(rtid),
        Json(AccessTokenResponse{
        access_token: access_token,
        expires_at: now + ACCESS_TOKEN_LIFETIME as i64
    })))
}








pub fn get_user_agent(headers: &HeaderMap) -> String {
    headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("NOT_PROVIDED").to_string()
}


