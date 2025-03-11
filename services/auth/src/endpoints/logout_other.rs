use axum::{body::Body, extract::State, http::{HeaderMap, Response, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::TokenEncoder, utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile}};
use tracing::info;
use crate::repository::tokens::hash_fingerprint;
use crate::{repository::{cookies::TokenCookie, tokens::{generate_access, generate_and_put_refresh}}, AppState};
use anyhow::Result;

use super::refresh::RefreshProcessor;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutBody {
    pub fingerprint: String
}

pub async fn logout_other(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<LogoutBody>,
) -> Result<Response<Body>, Response<Body>>  {
    Ok(RefreshProcessor::begin(jar, headers, state, payload.fingerprint)?.refresh_rules().await?.rm_all_refresh()?.generate_tokens()?)
}