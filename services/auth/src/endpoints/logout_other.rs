use axum::{body::Body, extract::State, http::{HeaderMap, Response}, Extension, Json};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::AppState;

use super::refresh::RefreshProcessor;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct LogoutBody {
//     pub fingerprint: String
// }

pub async fn logout_other(
    State(state): State<AppState>,
    jar: CookieJar,
    // headers: HeaderMap,
    Extension(user_info) : Extension<UserInfoExt>,

    // Json(payload): Json<LogoutBody>,
) -> Result<Response<Body>, Response<Body>>  {
    Ok(RefreshProcessor::begin(jar, &state, user_info).await?.refresh_rules().await?.rm_all_refresh().await?.generate_tokens().await?)
}