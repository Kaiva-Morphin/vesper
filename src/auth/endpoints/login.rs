use axum::{extract::State, Json};
use bcrypt::{hash, BcryptError, DEFAULT_COST};
use chrono::Utc;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};





#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    login: String,
    password: String,
    fingerprint: String,
}

use crate::{models::user_data::UserData, schema::user_data::dsl::*, shared::structs::tokens::tokens::AccessTokenResponse, AppState};
use diesel::prelude::*;


pub async fn login(
    State(state): State<AppState>,
    payload: Json<LoginBody>
) -> Result<Json<AccessTokenResponse>, StatusCode> {


    let login = payload.login.clone();
    let users: Vec<UserData> = state.postgre.interact(move |conn|{
        user_data.select(UserData::as_select()).filter(username.eq(&login).or(email.eq(&login))).load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }).await?;

    println!("{:?}",users);

    todo!()
}