use axum::{extract::State, http::HeaderMap, response::Redirect, Json};
use axum_extra::extract::CookieJar;
use bcrypt::{hash, BcryptError, DEFAULT_COST};
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{models::user_data::{CreateUserData, UserData}, shared::{errors::{adapt_error, AsStatusCode}, settings::*, structs::tokens::{cookies::TokenCookie, tokens::{TokenEncoder, AccessTokenPayload, AccessTokenResponse, RefreshTokenRecord}}}, AppState};

use crate::auth::checks::username::is_username_available;

use super::shared::{get_user_agent, process_tokens};



#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String,
    pub fingerprint: String
}

pub const PASSWORD_REGEX : &'static str = r"^[A-Za-z0-9_\-+=\$#~@*;:<>/\\|!]+$";
pub const EMAIL_REGEX : &'static str = r"^[^\s@]+@[^\s@]+\.[^\s@]+$";
pub const USERNAME_REGEX : &'static str = r"^[a-zA-Z0-9_]+$";

use lazy_static::lazy_static;

lazy_static!(
    pub static ref COMPILED_USERNAME_REGEX : Regex = Regex::new(USERNAME_REGEX).expect("Can't compile username regex!");
    pub static ref COMPILED_EMAIL_REGEX : Regex = Regex::new(EMAIL_REGEX).expect("Can't compile email regex!");
    pub static ref COMPILED_PASSWORD_REGEX : Regex = Regex::new(PASSWORD_REGEX).expect("Can't compile password regex!");
);

pub trait RegisterValidations {
    fn is_username_valid(&self) -> bool;
    fn is_email_valid(&self) -> bool;
    fn is_password_valid(&self) -> bool;
}

impl RegisterValidations for String {
    fn is_username_valid(&self) -> bool {
        let len = self.chars().count();
        COMPILED_USERNAME_REGEX.is_match(self) &&
            len >= MIN_LOGIN_LENGTH &&
            len <= MAX_LOGIN_LENGTH
    }
    fn is_email_valid(&self) -> bool {
        COMPILED_EMAIL_REGEX.is_match(self)
    }
    fn is_password_valid(&self) -> bool {
        let len = self.chars().count();
        COMPILED_PASSWORD_REGEX.is_match(self) &&
            len >= MIN_PASSWORD_LENGTH &&
            len <= MAX_PASSWORD_LENGTH
    }
}


impl RegisterBody {
    fn validate(&self) -> Result<(), StatusCode> {
        if self.username.is_username_valid() &&
            self.email.is_email_valid() &&
            self.password.is_password_valid() {return Ok(())}
        Err(StatusCode::UNAUTHORIZED)
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
            }
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct RegisterCriteria {
    username_len_max : usize,
    username_len_min : usize,
    password_len_max : usize,
    password_len_min : usize,
    username_regex : String,
    password_regex : String,
    email_regex: String,
}


pub async fn get_criteria() -> Json<RegisterCriteria> {
    Json(RegisterCriteria{
        username_len_max: MAX_LOGIN_LENGTH,
        username_len_min: MIN_LOGIN_LENGTH,
        password_len_max: MAX_PASSWORD_LENGTH,
        password_len_min: MIN_PASSWORD_LENGTH,
        username_regex: USERNAME_REGEX.to_string(),
        password_regex: PASSWORD_REGEX.to_string(),
        email_regex: EMAIL_REGEX.to_string(),
    })
}


pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(register_body): Json<RegisterBody>,
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
    //if !is_username_available(&state.postgre, register_body.username.clone()).await? {return Err(StatusCode::CONFLICT)};
    let _ = register_body.validate()?;
    let user = CreateUserData::from_register(&register_body)?;
    let user_uuid = user.uuid;
    let _ = state.postgre.interact(move |conn| {
        UserData::create(conn, &user).map_err(|e|{
            if let diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            })
    }).await?;
    process_tokens(jar, &state, user_uuid, register_body.fingerprint, get_user_agent(&headers)).await
}





