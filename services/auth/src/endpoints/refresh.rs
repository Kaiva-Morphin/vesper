use axum::{body::Body, extract::State, http::Response, Extension};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;

use crate::{repository::{refresh_processor::RefreshProcessor}, AppState};





pub async fn refresh_tokens(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(user_info) : Extension<UserInfoExt>,
) -> Result<Response<Body>, Response<Body>>  {
    RefreshProcessor::begin(jar, &state, user_info).await?.check_refresh_rules().await?.generate_tokens().await
}



