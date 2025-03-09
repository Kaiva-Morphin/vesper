use axum::{extract::State, http::{HeaderMap, StatusCode}, response::{IntoResponse, Redirect}, Json};
use axum_extra::extract::CookieJar;
use regex::Regex;
use serde::{Deserialize, Serialize};
use shared::{default_err, tokens::jwt::{AccessTokenResponse, RefreshTokenPayload, TokenEncoder}, utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile}};


use lazy_static::lazy_static;

use crate::{repository::tokens::{generate_access, generate_and_put_refresh}, AppState, CFG};

lazy_static!(
    pub static ref COMPILED_LOGIN_REGEX : Regex = Regex::new(format!(r"^([a-zA-Z0-9_]){{{},{}}}$", CFG.MIN_LOGIN_LENGTH, CFG.MAX_LOGIN_LENGTH).as_str()).expect("Can't compile login regex!");
    pub static ref COMPILED_EMAIL_REGEX : Regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("Can't compile email regex!");
    pub static ref COMPILED_PASSWORD_REGEX : Regex = Regex::new(format!(r"^([A-Za-z0-9_\-+=\$#~@*;:<>/\\|!]){{{},{}}}$", CFG.MIN_PASSWORD_LENGTH, CFG.MAX_PASSWORD_LENGTH).as_str()).expect("Can't compile password regex!");
);



// todo! Make shared!
pub trait RegisterValidations {
    fn is_login_valid(&self) -> bool;
    fn is_email_valid(&self) -> bool;
    fn is_password_valid(&self) -> bool;
    fn is_nickname_valid(&self) -> bool;
}

impl RegisterValidations for String {
    fn is_login_valid(&self) -> bool {
        COMPILED_LOGIN_REGEX.is_match(self)
    }
    fn is_email_valid(&self) -> bool {
        COMPILED_EMAIL_REGEX.is_match(self)
    }
    fn is_password_valid(&self) -> bool {
        COMPILED_PASSWORD_REGEX.is_match(self)
    }
    fn is_nickname_valid(&self) -> bool { 
        let len = self.trim().chars().count();
        len >= CFG.MIN_NICKNAME_LENGTH && len <= CFG.MAX_NICKNAME_LENGTH
    }
}






#[derive(Serialize, Deserialize)]
pub struct RegisterCriteria {
    login_len_max : usize,
    login_len_min : usize,
    password_len_max : usize,
    password_len_min : usize,
    username_regex : String,
    password_regex : String,
    email_regex: String,
}


pub async fn get_criteria() -> Json<RegisterCriteria> {
    Json(RegisterCriteria{
        login_len_max: CFG.MAX_LOGIN_LENGTH,
        login_len_min: CFG.MIN_LOGIN_LENGTH,
        password_len_max: CFG.MAX_PASSWORD_LENGTH,
        password_len_min: CFG.MIN_PASSWORD_LENGTH,
        username_regex: COMPILED_LOGIN_REGEX.to_string(),
        password_regex: COMPILED_PASSWORD_REGEX.to_string(),
        email_regex: COMPILED_EMAIL_REGEX.to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterBody {
    pub login: String,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub email_code: String,
    pub turnstile_response: String,
    pub fingerprint: String,
    pub tos_accepted: bool
}

impl RegisterBody {
    fn validate(&self) -> Result<(), AppErr> {
        if self.login.is_login_valid() &&
            self.nickname.is_nickname_valid() &&
            self.email.is_email_valid() &&
            self.password.is_password_valid() {return Ok(())}
        Err(AppErr::default())
    }
}


pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(request_body): Json<RegisterBody>,
) -> Result<impl IntoResponse, AppErr> {
    if !request_body.tos_accepted {return Ok((StatusCode::BAD_REQUEST, "Accept ToS!").into_response())};
    let Ok(_) = request_body.validate() else {return Ok((StatusCode::BAD_REQUEST, "Invalid data sent!").into_response())};
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_response.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    #[cfg(not(feature = "disable_email"))]
    if !state.verify_register_code(request_body.email_code.clone(), request_body.email.clone())? {return Ok((StatusCode::BAD_REQUEST, "Invalid email code!").into_response())};
    let fingerprint = request_body.fingerprint.clone();
    let email = request_body.email.clone();
    let r = state.register_user(request_body).await?;
    let Ok((user_id, rules)) = r else {
        return Ok((StatusCode::CONFLICT, r.err().unwrap()).into_response())
    };
    let jar = generate_and_put_refresh(jar, &state, &user_id, fingerprint, get_user_agent(&headers), get_user_ip(&headers), email, rules)?;
    let access_response = generate_access(user_id)?;
    Ok((jar, access_response).into_response())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestCodeBody {
    pub email: String,
    pub turnstile_response: String,
}

pub async fn request_register_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request_body): Json<RequestCodeBody>,
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_response.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::UNAUTHORIZED, "Turnstile failed").into_response())};
    state.send_register_code(&request_body.email).await?;
    Ok("Code sent".into_response())
}