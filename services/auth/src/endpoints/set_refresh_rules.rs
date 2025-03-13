use axum::{body::Body, extract::State, http::{HeaderMap, Response}, Extension, Json};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use serde::{Deserialize, Serialize};
use shared::tokens::jwt::RefreshRules;

use crate::AppState;

use super::refresh::RefreshProcessor;



#[derive(Serialize, Deserialize)]
pub struct SetRefreshRules {
    pub allow_suspicious_refresh : bool,
    pub warn_suspicious_refresh : bool
}

pub async fn set_refresh_rules(
    State(state): State<AppState>,
    jar: CookieJar,
    // headers: HeaderMap,
    Extension(user_info) : Extension<UserInfoExt>,
    Json(SetRefreshRules{
        allow_suspicious_refresh,
        warn_suspicious_refresh,
    }): Json<SetRefreshRules>,
) -> Result<Response<Body>, Response<Body>> {
    let new_rules= RefreshRules { warn_suspicious_refresh, allow_suspicious_refresh};
    Ok(RefreshProcessor::begin(jar, state, user_info)?.refresh_rules().await?.update_refresh_rules(new_rules).await?.generate_tokens()?)
}



