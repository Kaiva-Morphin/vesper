use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::TokenEncoder, utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile}};
use tracing::info;
use crate::repository::tokens::hash_fingerprint;
use crate::{repository::{cookies::TokenCookie, tokens::{generate_access, generate_and_put_refresh}}, AppState};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutBody {
    pub fingerprint: String
}

pub async fn logout_other(
    State(state): State<AppState>,
    mut jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<LogoutBody>,
) -> Result<impl IntoResponse, AppErr>  {
    let Some(refresh_token_string) = jar.get_refresh() else {return Ok((jar.rm_refresh(), StatusCode::UNAUTHORIZED).into_response())};
    jar = jar.rm_refresh();
    let Some(refresh_payload) = TokenEncoder::decode_refresh(refresh_token_string) else {return Ok((jar, StatusCode::UNAUTHORIZED).into_response())};
    let record = state.redis.pop_refresh(refresh_payload.rtid)?;
    let Some(record) = record else {return Ok((jar, StatusCode::INTERNAL_SERVER_ERROR).into_response())};
    let user_agent = get_user_agent(&headers);
    let user_ip = get_user_ip(&headers);
    if refresh_payload.rules.warn_suspicious_refresh {
        if record.fingerprint != payload.fingerprint.clone() ||
            record.user_agent != user_agent {
                state.send_suspicious_refresh(&record.email, user_agent.clone(), user_ip.clone()).await?;
                info!("Fingerprint: {} != {} || user agent: {} != {}", hash_fingerprint(&record.fingerprint), hash_fingerprint(&payload.fingerprint), record.user_agent, user_agent);
                if !refresh_payload.rules.allow_suspicious_refresh {return Ok((StatusCode::UNAUTHORIZED, jar, "Blocked due refresh rules").into_response());}
            };
    }
    state.redis.rm_all_refresh(refresh_payload.user)?;
    let jar = generate_and_put_refresh(jar, &state, &record.user, payload.fingerprint, user_agent, user_ip, record.email, refresh_payload.rules)?;
    let access_response = generate_access(record.user)?;
    Ok((StatusCode::OK, jar, access_response).into_response())
}