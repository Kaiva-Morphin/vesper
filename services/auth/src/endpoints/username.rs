use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};
use shared::utils::app_err::AppErr;


use crate::AppState;
use anyhow::Result;


#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUserUID {
    pub user_uid: String
}

// todo: all usernames in redis?
pub async fn check_user_uid(
    State(state): State<AppState>,
    Query(params): Query<CheckUserUID>,
) -> Result<Json<bool>, AppErr> {
    Ok(Json(state.is_uid_available(params.user_uid.to_lowercase()).await?))
}