use axum::{extract::State, Json};
use bcrypt::{hash, BcryptError, DEFAULT_COST};
use chrono::Utc;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{models::user_data::{CreateUserData, UserData}, shared::{checks::UsernameValidation, errors::{adapt_error, AsStatusCode}, settings::*, structs::tokens::TokenPair}, AppState};

use crate::auth::checks::username::is_username_available;




#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String
}

impl RegisterBody {
    fn validate(&self) -> Result<(), (StatusCode, Json<Vec<String>>)> {
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
        Err((StatusCode::BAD_REQUEST, Json(errors)))
    }
}

impl AsStatusCode for BcryptError {
    fn as_interaction_error(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl CreateUserData {
    pub fn from_register(payload: &RegisterBody) -> Result<CreateUserData, StatusCode> {
        let hashed = hash(payload.password.clone(), DEFAULT_COST).map_err(adapt_error)?;
        Ok(
        CreateUserData{
            uuid: Uuid::new_v4(),
            username: payload.username.clone(),
            password: hashed,
            nickname: payload.username.clone(),
            email: payload.email.clone(),
            created: Utc::now().timestamp(),
            discord_id: None,
            google_id: None
        })
    }
}

pub async fn register(
    State(state): State<AppState>,
    payload: Json<RegisterBody>
) -> Result<Json<TokenPair>, (StatusCode, Json<Vec<String>>)> {
    if !is_username_available(&state.postgre, payload.username.clone()).await
        .map_err(|v| (v, Json(vec![])))? {return Err((StatusCode::CONFLICT, Json(vec!["username taken".to_string()])))};
    let _ = payload.validate()?;
    let user = CreateUserData::from_register(&payload).map_err(|v| (v, Json(vec![])))?;
    let rtid = Uuid::new_v4();
    let tokens = TokenPair::generate_pair(user.uuid.clone(), rtid).map_err(|v| (v, Json(vec![])))?;
    state.tokens.set_ex(rtid.to_string(), rtid.to_string(), REFRESH_TOKEN_LIFETIME).await.map_err(|v| (v, Json(vec![])))?;
    let _ = state.postgre.interact(move |conn| {
        UserData::create(conn, &user).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }).await.map_err(|v| (v, Json(vec![])))?;
    Ok(Json(tokens))
}





