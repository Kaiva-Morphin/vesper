use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use serde::{Deserialize, Serialize};
use shared::utils::app_err::AppErr;

use crate::{repository::tokens::{generate_access, generate_and_put_refresh}, AppState};
use anyhow::Result;

#[cfg(not(feature = "disable_turnstile"))]
use shared::utils::{header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
    pub turnstile_token: String
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(user_info) : Extension<UserInfoExt>,
    // headers: HeaderMap,
    Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(login_body.turnstile_token.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::UNAUTHORIZED, "Turnstile failed").into_response())};
    let guid = state.login(&login_body).await?;
    let Some((guid, settings)) = guid else {return Ok((StatusCode::UNAUTHORIZED, "Incorrect credentials!").into_response())};
    // let Some(email) = state.get_email_from_login_cred(&login_body.email).await? else {return Ok((StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response())};
    state.send_new_login(login_body.email.clone(), user_info.ip.clone(), user_info.user_agent.clone()).await?; // TODO!: ADD TRUSTED USER DEVICES AND 2FA
    let jar = generate_and_put_refresh(jar, &state, &guid, user_info, login_body.email, settings).await?;
    let access_response = generate_access(guid)?;
    Ok((jar, access_response).into_response())
}