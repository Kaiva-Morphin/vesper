use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::{AccessTokenResponse, RefreshRules, TokenEncoder}, utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}}};
use tracing::warn;

use crate::{repository::{cookies::TokenCookie, tokens::{generate_access, generate_and_put_refresh}}, AppState};



#[derive(Serialize, Deserialize)]
pub struct SetRefreshRules {
    pub allow_suspicious_refresh : bool,
    pub warn_suspicious_refresh : bool,
    pub fingerprint: String
}

pub async fn set_refresh_rules(
    State(state): State<AppState>,
    mut jar: CookieJar,
    headers: HeaderMap,
    Json(SetRefreshRules{
        allow_suspicious_refresh,
        warn_suspicious_refresh,
        fingerprint,
    }): Json<SetRefreshRules>,
) -> Result<impl IntoResponse, AppErr>  {
    let Some(refresh_token_string) = jar.get_refresh() else {return Ok((jar.rm_refresh(), StatusCode::UNAUTHORIZED).into_response())};
    jar = jar.rm_refresh();
    let Some(refresh_payload) = TokenEncoder::decode_refresh(refresh_token_string) else {return Ok((jar, StatusCode::UNAUTHORIZED).into_response())};
    let record = state.redis.pop_refresh(refresh_payload.rtid)?;
    let Some(record) = record else {return Ok((jar, StatusCode::INTERNAL_SERVER_ERROR).into_response())};
    let user_agent = get_user_agent(&headers);
    let user_ip = get_user_ip(&headers);
    if !refresh_payload.rules.allow_suspicious_refresh {
        if record.fingerprint != fingerprint.clone() ||
            record.user_agent != user_agent { state.send_suspicious_refresh(&record.email, user_agent.clone(), user_ip.clone()).await?; };
    }
    state.update_refresh_rules(&record.email, user_ip.clone(), user_agent.clone(), RefreshRules{warn_suspicious_refresh, allow_suspicious_refresh}).await?;
    let jar = generate_and_put_refresh(jar, &state, &record.user, fingerprint, user_agent, user_ip, record.email, refresh_payload.rules)?;
    let access_response = generate_access(record.user)?;
    Ok((StatusCode::OK, jar, access_response).into_response())
}



