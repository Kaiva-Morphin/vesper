use axum::{extract::State, http::HeaderMap, response::Redirect, Json};
use axum_extra::extract::CookieJar;
use bcrypt::{hash, BcryptError, DEFAULT_COST};
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{models::user_data::{CreateUserData, UserData}, shared::{errors::{adapt_error, AsStatusCode}, settings::*, structs::tokens::{cookies::TokenCookie, tokens::{AccessTokenEncoder, AccessTokenPayload, AccessTokenResponse, RefreshTokenRecord}}}, AppState};

use crate::auth::checks::username::is_username_available;

use super::shared::get_user_agent;



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

pub trait UsernameValidation {
    fn is_username_valid(&self) -> bool;
}

impl UsernameValidation for String {
    fn is_username_valid(&self) -> bool {
        let re = Regex::new(USERNAME_REGEX).unwrap();
        re.is_match(self)
    }
}

impl RegisterBody {
    fn validate(&self) -> Result<(), StatusCode> {
        if !self.username.is_username_valid(){return Err(StatusCode::UNAUTHORIZED);}
        
        let re = Regex::new(PASSWORD_REGEX).unwrap();
        if !re.is_match(self.password.as_str()){return Err(StatusCode::UNAUTHORIZED);}
        // Password must include only latin symbols, numbers and '_-+=$#~@*;:<>/\\|'
        let re = Regex::new(EMAIL_REGEX).unwrap();
        if !re.is_match(self.email.as_str()) {return Err(StatusCode::UNAUTHORIZED);}

        let username_len = self.username.chars().count();
        if username_len < MIN_USERNAME_LENGTH {return Err(StatusCode::UNAUTHORIZED);}
        if username_len > MAX_USERNAME_LENGTH {return Err(StatusCode::UNAUTHORIZED);}
        let password_len = self.password.chars().count();
        if password_len < MIN_PASSWORD_LENGTH {return Err(StatusCode::UNAUTHORIZED);}
        if password_len > MAX_PASSWORD_LENGTH {return Err(StatusCode::UNAUTHORIZED);}
        Ok(())
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
        username_len_max: MAX_USERNAME_LENGTH,
        username_len_min: MIN_USERNAME_LENGTH,
        password_len_max: MAX_PASSWORD_LENGTH,
        password_len_min: MIN_PASSWORD_LENGTH,
        username_regex: USERNAME_REGEX.to_string(),
        password_regex: PASSWORD_REGEX.to_string(),
        email_regex: EMAIL_REGEX.to_string(),
    })
}


// todo: do not use Vec<String>!
pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    payload: Json<RegisterBody>,
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
    if !is_username_available(&state.postgre, payload.username.clone()).await? {return Err(StatusCode::CONFLICT)};
    let _ = payload.validate()?;
    let user = CreateUserData::from_register(&payload)?;
    let rtid = Uuid::new_v4();
    
    let token_record = RefreshTokenRecord{
        rtid,
        user: user.uuid,
        fingerprint: payload.fingerprint.clone(),
        user_agent: get_user_agent(&headers),
        ip: "Undefined".to_string() // TODO! can be provided after nginx????  println!("\n\n\n{:?}", headers.get("x-forwarded-for"));  println!("\n\n\n{:?}", headers.get("x-real-ip"));
    };
    
    let access_payload = AccessTokenPayload{
        user: user.uuid,
        created: Utc::now().timestamp(),
        lifetime: ACCESS_TOKEN_LIFETIME
    };

    let access_token = AccessTokenEncoder::encode(access_payload)?;
    let _ = state.postgre.interact(move |conn| {
        UserData::create(conn, &user).map_err(|e|{
            if let diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            })
    }).await?;
    state.tokens.set_refresh(token_record)?;

    Ok((
        jar.put_rtid(rtid),
        Json(AccessTokenResponse{
        access_token: access_token,
        expires_at: Utc::now().timestamp() + ACCESS_TOKEN_LIFETIME as i64
    })))
}





