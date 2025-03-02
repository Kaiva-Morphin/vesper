use axum::{extract::State, http::HeaderMap, Json};
use axum_extra::extract::CookieJar;
use bcrypt::{hash, BcryptError, DEFAULT_COST};
use chrono::Utc;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use ::uuid::Uuid;
use crate::{auth::endpoints::shared::get_user_agent, models::user_data::UserData, schema::user_data::dsl::*, shared::{settings::ACCESS_TOKEN_LIFETIME, structs::tokens::{cookies::TokenCookie, tokens::{TokenEncoder, AccessTokenPayload, AccessTokenResponse, RefreshTokenRecord}}}, AppState};
use diesel::prelude::*;

use super::shared::process_tokens;





#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    login: String,
    password: String,
    fingerprint: String,
}



pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(login_body): Json<LoginBody>
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
    
    let users: Vec<UserData> = state.postgre.interact(move |conn|{
        user_data.select(UserData::as_select()).filter(username.eq(&login_body.login).or(email.eq(&login_body.login))).load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }).await?;

    let user = users.first().ok_or(StatusCode::UNAUTHORIZED)?;
    if !bcrypt::verify(login_body.password, user.password.as_str()).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::UNAUTHORIZED);
    }
    process_tokens(jar, &state, user.uuid, login_body.fingerprint, get_user_agent(&headers)).await
}