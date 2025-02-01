use axum::{extract::State, http::HeaderMap, Json};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::endpoints::shared::get_user_agent, shared::{settings::ACCESS_TOKEN_LIFETIME, structs::tokens::{cookies::TokenCookie, tokens::{AccessTokenEncoder, AccessTokenPayload, AccessTokenResponse}}}, AppState};

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
    
    let Ok(rtid) = jar.get_rtid() else {return Err((jar.rm_rtid(), StatusCode::UNAUTHORIZED))};   
    let record = state.tokens.pop_refresh(rtid);
    let Ok(record) = record else {return Err((jar, StatusCode::INTERNAL_SERVER_ERROR))};
    let jar = jar.rm_rtid();
    let Some(mut record) = record else {return Err((jar, StatusCode::UNAUTHORIZED))};

    if  record.fingerprint == payload.fingerprint.clone() &&
    record.user_agent == get_user_agent(&headers) {
        let now = Utc::now().timestamp();
        let rtid = Uuid::new_v4();
        record.rtid = rtid;
        let access_payload = AccessTokenPayload{
            user: record.user,
            created: now,
            lifetime: ACCESS_TOKEN_LIFETIME
        };
        let Ok(access_token) = AccessTokenEncoder::encode(access_payload) else {return Err((jar, StatusCode::UNAUTHORIZED))};
        state.tokens.set_refresh(record).map_err(|v|(jar.clone(), v))?;
        Ok((
            jar.put_rtid(rtid),
            Json(AccessTokenResponse{
            access_token: access_token,
            expires_at: now + ACCESS_TOKEN_LIFETIME as i64
        })))
    } else {
        Err((jar, StatusCode::UNAUTHORIZED)) 
    }
}