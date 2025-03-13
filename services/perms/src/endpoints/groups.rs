use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile};

use crate::AppState;
use anyhow::Result;


pub async fn get(
    State(state): State<AppState>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    Ok(().into_response())
}

pub async fn post(
    State(state): State<AppState>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    Ok(().into_response())
}

pub async fn put(
    State(state): State<AppState>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    Ok(().into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    // headers: HeaderMap,
    // Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    Ok(().into_response())
}