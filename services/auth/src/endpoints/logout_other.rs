use axum::{body::Body, extract::State, http::{HeaderMap, Response}, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::AppState;

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
    Ok(RefreshProcessor::begin(jar, &headers, state, payload.fingerprint)?.refresh_rules().await?.rm_all_refresh()?.generate_tokens()?)
}