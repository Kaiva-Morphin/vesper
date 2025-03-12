use axum::{body::Body, extract::State, http::{HeaderMap, Response}, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::tokens::jwt::RefreshRules;

use crate::AppState;

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
    Ok(RefreshProcessor::begin(jar, &headers, state, fingerprint)?.refresh_rules().await?.update_refresh_rules(new_rules).await?.generate_tokens()?)
}



