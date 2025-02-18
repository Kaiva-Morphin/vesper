use axum::{extract::State, http::HeaderMap, Json};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::endpoints::shared::get_user_agent, shared::{settings::{ACCESS_TOKEN_LIFETIME, REFRESH_TOKEN_LIFETIME}, structs::tokens::{cookies::TokenCookie, tokens::{AccessTokenPayload, AccessTokenResponse, RefreshTokenPayload, TokenEncoder}}}, AppState};

#[derive(Serialize, Deserialize)]
pub struct RefreshBody {
    pub fingerprint: String
}


pub async fn refresh_tokens(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    payload: Json<RefreshBody>,
) -> Result<(CookieJar, Json<AccessTokenResponse>), (CookieJar, StatusCode)> {
    let Ok(refresh_token_string) = jar.get_refresh() else {return Err((jar.rm_refresh(), StatusCode::UNAUTHORIZED))};
    let Ok(mut refresh_payload) = TokenEncoder::decode_refresh(refresh_token_string) else {return Err((jar, StatusCode::UNAUTHORIZED))};
    let record = state.tokens.pop_refresh(refresh_payload.rtid);
    let Ok(record) = record else {return Err((jar, StatusCode::INTERNAL_SERVER_ERROR))};
    let jar = jar.rm_refresh();
    let Some(mut record) = record else {return Err((jar, StatusCode::UNAUTHORIZED))};

    if  !(record.fingerprint == payload.fingerprint.clone() &&
    record.user_agent == get_user_agent(&headers)) { return Err((jar, StatusCode::UNAUTHORIZED)) };
    
    let now = Utc::now().timestamp();
    let rtid = Uuid::new_v4();
    record.rtid = rtid;
    
    refresh_payload.rtid = rtid;
    refresh_payload.exp = Utc::now().timestamp() + REFRESH_TOKEN_LIFETIME as i64;

    state.tokens.set_refresh(record).map_err(|v|(jar.clone(), v))?;
    let access_payload = AccessTokenPayload{
        user: refresh_payload.user,
        exp: now + ACCESS_TOKEN_LIFETIME as i64
    };
    let Ok(access_token) = TokenEncoder::encode_access(access_payload) else {return Err((jar, StatusCode::UNAUTHORIZED))};
    let Ok(refresh_token) = TokenEncoder::encode_refresh(refresh_payload) else {return Err((jar, StatusCode::UNAUTHORIZED))};
    Ok((
        jar.put_refresh(refresh_token),
        Json(AccessTokenResponse{
        access_token: access_token,
        exp: now + ACCESS_TOKEN_LIFETIME as i64
    })))
}

