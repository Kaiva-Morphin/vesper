use axum::{http::HeaderMap, Json};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{shared::{settings::ACCESS_TOKEN_LIFETIME, structs::tokens::{cookies::TokenCookie, tokens::{AccessTokenPayload, AccessTokenResponse, RefreshTokenPayload, RefreshTokenRecord, TokenEncoder}}}, AppState};









pub async fn process_tokens(
    jar: CookieJar,
    state: &AppState,
    user: Uuid,
    fingerprint: String,
    user_agent: String,
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
    let rtid: Uuid = Uuid::new_v4();
    let refresh_record = RefreshTokenRecord{
        rtid,
        user,
        fingerprint,
        user_agent,
        ip: "Undefined".to_string() // TODO! can be provided after nginx????  println!("\n\n\n{:?}", headers.get("x-forwarded-for"));  println!("\n\n\n{:?}", headers.get("x-real-ip"));
    };

    let refresh_payload = RefreshTokenPayload{
        rtid,
        user,
        exp: Utc::now().timestamp() + ACCESS_TOKEN_LIFETIME as i64
    };

    let now = Utc::now().timestamp();
    
    let access_payload = AccessTokenPayload{
        user,
        exp: now + ACCESS_TOKEN_LIFETIME as i64
    };

    let access_token = TokenEncoder::encode_access(access_payload).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let refresh_token = TokenEncoder::encode_refresh(refresh_payload).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.tokens.set_refresh(refresh_record)?;

    Ok((
        jar.put_refresh(refresh_token),
        Json(AccessTokenResponse{
        access_token: access_token,
        exp: now + ACCESS_TOKEN_LIFETIME as i64
    })))
}

pub fn get_user_agent(headers: &HeaderMap) -> String {
    headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("NOT_PROVIDED").to_string()
}


