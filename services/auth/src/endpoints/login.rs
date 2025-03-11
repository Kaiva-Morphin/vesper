use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use shared::utils::{app_err::AppErr, header::{get_user_agent, get_user_ip}, verify_turnstile::verify_turnstile};

use crate::{repository::tokens::{generate_access, generate_and_put_refresh}, AppState};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    pub email_or_login: String,
    pub password: String,
    pub fingerprint: String,
    pub turnstile_response: String
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(login_body): Json<LoginBody>
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(login_body.turnstile_response.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::UNAUTHORIZED, "Turnstile failed").into_response())};
    let user_id = state.login(&login_body).await?;
    let Some((user_id, settings)) = user_id else {return Ok((StatusCode::UNAUTHORIZED, "Incorrect credentials!").into_response())};
    let Some(email) = state.get_email_from_login_cred(&login_body.email_or_login).await? else {return Ok((StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response())};
    let ip = get_user_ip(&headers);
    let ua = get_user_agent(&headers);
    state.send_new_login(&email, ip.clone(), ua.clone()).await?; //TODO!: ADD TRUSTED USER DEVICES 

    let jar = generate_and_put_refresh(jar, &state, &user_id, login_body.fingerprint, ua, ip, email, settings)?;
    let access_response = generate_access(user_id)?;
    Ok((jar, access_response).into_response())
}