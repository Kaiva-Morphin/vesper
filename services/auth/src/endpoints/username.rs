use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use shared::utils::app_err::AppErr;


use crate::AppState;
use anyhow::Result;


#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUsername {
    pub username: String
}

pub async fn check_username(
    State(state): State<AppState>,
    Json(CheckUsername{username}): Json<CheckUsername>
) -> Result<Json<bool>, AppErr> {
    Ok(Json(state.is_login_available(username).await?))
}