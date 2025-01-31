use axum::{extract::State, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::checks::username::is_username_available;
use crate::models::user_data::UserData;


use crate::schema::user_data::dsl::*;
use crate::shared::structs::app_state::postgre::Postgre;
use crate::AppState;

use diesel::prelude::*;


#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUsername {
    pub username: String
}

pub async fn check_username(
    State(state): State<AppState>,
    payload: Json<CheckUsername>
) -> Result<Json<bool>, StatusCode> {
    Ok(Json(is_username_available(&state.postgre, payload.username.clone()).await?))
}