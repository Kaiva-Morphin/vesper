use axum::{
    extract::State, http::StatusCode, response::IntoResponse, Json
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use regex::Regex;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use std::sync::Arc;

use crate::{AppState, CONTACT_ADMIN_MESSAGE, MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH, MIN_PASSWORD_LENGTH, MIN_USERNAME_LENGTH};

use super::shared::{create_token_record, Tokens, UserData, UsernameValidation};



#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String
}

impl RegisterBody {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        if !self.username.is_username_valid(){errors.push("Username must include only latin symbols, numbers and _".to_string())}
        
        let re = Regex::new(r"^[A-Za-z0-9_\-+=\$#~@*;:<>/\\|!]+$").unwrap();
        if !re.is_match(self.password.as_str()){errors.push("Password must include only latin symbols, numbers and '_-+=$#~@*;:<>/\\|'".to_string())}

        let re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !re.is_match(self.email.as_str()) {errors.push(format!("Email is incorrect!"))}

        let username_len = self.username.chars().count();
        if username_len < MIN_USERNAME_LENGTH {errors.push(format!("Username length must be at least {}! Yours is {}", MIN_USERNAME_LENGTH, username_len))}
        if username_len > MAX_USERNAME_LENGTH {errors.push(format!("Username length cant be longer than {}! Yours is {}", MAX_USERNAME_LENGTH, username_len))}
        let password_len = self.password.chars().count();
        if password_len < MIN_PASSWORD_LENGTH {errors.push(format!("Password length must be at least {}! Yours is {}", MIN_PASSWORD_LENGTH, password_len))}
        if password_len > MAX_PASSWORD_LENGTH {errors.push(format!("Password length must be at least {}! Yours is {}", MAX_PASSWORD_LENGTH, password_len))}
        if errors.is_empty(){return Ok(())}
        Err(errors)
    }
}



impl UserData {
    pub fn from_register(payload: &RegisterBody) -> Result<(Tokens, UserData), String> {
        let Ok(hashed) = hash(payload.password.clone(), DEFAULT_COST) else {return Err("Server cant hash password!".to_string())};
        let Ok(tokens) = Tokens::get_pair(payload.username.clone()) else {return Err("Server cant generate tokens!".to_string())};
        Ok((
        tokens,
        UserData{
            username: payload.username.clone(),
            password: hashed,
            nickname: payload.username.clone(),
            email: payload.email.clone(),
            created: Utc::now().timestamp() as u64
        }))
    }
}

pub async fn register(
    State(appstate): State<Arc<AppState>>,
    payload: Option<Json<RegisterBody>>
) -> impl IntoResponse {
    // if payload is in incorrect form
    let Some(payload) = payload else { return (StatusCode::BAD_REQUEST, "Incorrect form sent!").into_response()};

    if let Err(e) = payload.validate() {return (StatusCode::BAD_REQUEST, Json(e)).into_response()};

    appstate.db.execute("");


    let Ok(user_data) = db.select::<Option<UserData>>(("user_data", &payload.username)).await 
    else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant get data from db! {}", CONTACT_ADMIN_MESSAGE)).into_response()};

    if user_data.is_some() {return (StatusCode::CONFLICT, "Username taken").into_response();};

    let res = UserData::from_register(&payload);
    let Ok((tokens, user_data)) = res
    else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("{} {}", res.unwrap_err(), CONTACT_ADMIN_MESSAGE)).into_response()};

    let data: Result<Option<UserData>, surrealdb::Error> = db.create(("user_data", &payload.username)).content(user_data).await;
    let Ok(_) = data else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant save data to db! {}", CONTACT_ADMIN_MESSAGE)).into_response()};

    if let Err(r) = create_token_record(&db, payload.username.clone(), tokens.refresh.clone()).await {
        return r;
    }

    (StatusCode::OK, Json(tokens)).into_response()
}