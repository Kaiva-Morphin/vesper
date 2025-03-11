use axum::{body::Body, extract::State, http::{HeaderMap, Response, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::{AccessTokenResponse, RefreshRules, TokenEncoder}, utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}}};
use tracing::warn;

use crate::{repository::{cookies::TokenCookie, tokens::{generate_access, generate_and_put_refresh}}, AppState};

use super::refresh::RefreshProcessor;



#[derive(Serialize, Deserialize)]
pub struct SetRefreshRules {
    pub allow_suspicious_refresh : bool,
    pub warn_suspicious_refresh : bool,
    pub fingerprint: String
}

pub async fn set_refresh_rules(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(SetRefreshRules{
        allow_suspicious_refresh,
        warn_suspicious_refresh,
        fingerprint,
    }): Json<SetRefreshRules>,
) -> Result<Response<Body>, Response<Body>> {
    let new_rules= RefreshRules { warn_suspicious_refresh, allow_suspicious_refresh};
    Ok(RefreshProcessor::begin(jar, headers, state, fingerprint)?.refresh_rules().await?.update_refresh_rules(new_rules).await?.generate_tokens()?)
}



