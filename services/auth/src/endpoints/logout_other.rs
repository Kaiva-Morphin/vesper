use axum::{body::Body, extract::State, http::Response, Extension};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use anyhow::Result;

use crate::{repository::refresh_processor::RefreshProcessor, AppState};


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
    RefreshProcessor::begin(jar, &state, user_info).await?
            .check_refresh_rules().await?
            .rm_all_refresh().await?
            .generate_tokens().await
}