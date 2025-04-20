use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Extension, Json};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use shared::utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, validation::{RegisterValidations, COMPILED_EMAIL_REGEX, COMPILED_LOGIN_REGEX, COMPILED_PASSWORD_REGEX}, verify_turnstile::verify_turnstile};


use lazy_static::lazy_static;

use crate::{repository::tokens::{generate_access, generate_and_put_refresh}, AppState, CFG};


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
    Extension(user_info) : Extension<UserInfoExt>,
    Json(request_body): Json<RegisterBody>,
) -> Result<impl IntoResponse, AppErr> {
    if !request_body.tos_accepted {return Ok((StatusCode::BAD_REQUEST, "Accept ToS!").into_response())}; // hehehe~
    let Ok(_) = request_body.validate() else {return Ok((StatusCode::BAD_REQUEST, "Invalid data sent!").into_response())};
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_response.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    #[cfg(not(feature = "disable_email"))]
    if !state.verify_register_code(request_body.email_code.clone(), request_body.email.clone()).await? {return Ok((StatusCode::BAD_REQUEST, "Invalid email code!").into_response())};
    let email = request_body.email.clone();
    let r = state.register_user(request_body).await?;
    let Ok((user_id, rules)) = r else {
        return Ok((StatusCode::CONFLICT, r.err().unwrap()).into_response())
    };
    let jar = generate_and_put_refresh(jar, &state, &user_id, user_info, email, rules).await?;
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