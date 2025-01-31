use axum::{extract::State, Json};
use bcrypt::{hash, BcryptError, DEFAULT_COST};
use chrono::Utc;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{models::user_data::{CreateUserData, UserData}, shared::{checks::UsernameValidation, errors::{adapt_error, AsStatusCode}, settings::*, structs::tokens::TokenPair}, AppState};




#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    login: String,
    password: String
}

use crate::schema::user_data::dsl::*;
use diesel::prelude::*;


pub async fn login(
    State(state): State<AppState>,
    payload: Json<LoginBody>
) -> Result<Json<TokenPair>, StatusCode> {


    let login = payload.login.clone();
    let users: Vec<UserData> = state.postgre.interact(move |conn|{
        user_data.select(UserData::as_select()).filter(username.eq(&login).or(email.eq(&login))).load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }).await?;

    println!("{:?}",users);

    todo!()
}