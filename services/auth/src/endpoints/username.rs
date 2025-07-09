use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use shared::utils::app_err::AppErr;


use crate::AppState;
use anyhow::Result;


#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUserUID {
    pub user_uid: String
}

pub async fn check_user_uid(
    State(state): State<AppState>,
    Json(CheckUserUID{user_uid}): Json<CheckUserUID>
) -> Result<Json<bool>, AppErr> {
    Ok(Json(state.is_login_available(user_uid).await?))
}