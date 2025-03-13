use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, Extension, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::AccessTokenPayload, utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile}};
use tracing::info;

use crate::AppState;
use anyhow::Result;




pub async fn get(
    State(state): State<AppState>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    Ok(("OK").into_response())
}

pub async fn get_authed(
    State(state): State<AppState>,
    Extension(token): Extension<AccessTokenPayload>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    info!("{token:#?}");
    Ok(("OK").into_response())
}


pub async fn get_perm(
    State(state): State<AppState>,
    Path(p) : Path<Vec<(String, String)>>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    Ok((Json(p)).into_response())
}