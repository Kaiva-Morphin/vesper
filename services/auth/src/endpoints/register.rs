use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Extension, Json};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use serde::{Deserialize, Serialize};
use shared::utils::{app_err::AppErr, validation::RegisterValidations};



use crate::{repository::tokens::{generate_access, generate_and_put_refresh}, AppState};

#[cfg(not(feature = "disable_turnstile"))]
use shared::utils::{header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterBody {
    pub uid: String,
    pub nickname: String,
    pub email: String,
    pub password: String,
    pub email_code: String,
    pub turnstile_token: String,
    pub tos_accepted: bool
}

impl RegisterBody {
    fn validate(&self) -> Result<(), &'static str> {
        if !self.uid.is_uid_valid() {return Err("Invalid uid!")}
        if !self.nickname.is_nickname_valid() {return Err("Invalid nickname!")}
        if !self.email.is_email_valid() {return Err("Invalid email!")}
        if !self.password.is_password_valid() {return Err("Invalid password!")}
        Ok(())
    }
}


pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(user_info) : Extension<UserInfoExt>,
    Json(request_body): Json<RegisterBody>,
) -> Result<impl IntoResponse, AppErr> {
    if !request_body.tos_accepted {return Ok((StatusCode::BAD_REQUEST, "Accept ToS!").into_response())}; // hehehe~
    if let Err(msg) = request_body.validate() {return Ok((StatusCode::BAD_REQUEST, msg).into_response())};
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_token.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    #[cfg(not(feature = "disable_email"))]
    if !state.verify_register_code(request_body.email_code.clone(), request_body.email.clone()).await? {return Ok((StatusCode::BAD_REQUEST, "Invalid email code!").into_response())};
    
    let email = request_body.email.clone();
    let r = state.register_user(
        request_body.uid,
        request_body.nickname,
        request_body.email,
        request_body.password,
        None,
        None
    ).await?;
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
    pub turnstile_token: String,
}

pub async fn request_register_code(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(request_body): Json<RequestCodeBody>,
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_token.clone(), get_user_ip(&_headers)).await {return Ok((StatusCode::UNAUTHORIZED, "Turnstile failed").into_response())};
    state.send_register_code(request_body.email).await?;
    Ok("Code sent".into_response())
}