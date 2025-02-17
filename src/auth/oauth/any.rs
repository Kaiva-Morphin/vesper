use std::collections::HashMap;

use axum::{extract::{Query, State}, http::HeaderMap, response::{IntoResponse, Redirect}, Json};
use axum_extra::extract::CookieJar;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use ::uuid::Uuid;

use crate::{auth::{endpoints::{register::RegisterValidations, shared::{get_user_agent, process_tokens}}, oauth::{discord::{discord_callback, fetch_discord_user_info}, google::{fetch_google_user_info, google_callback}, shared::{UserInfo, DISCORD_PROVIDER, GOOGLE_PROVIDER}}}, models::user_data::{CreateUserData, UserData}, shared::{env::TEMPORARY_USERDATA_TOKEN_SECRET, errors::adapt_error, settings::TEMPORARY_USERDATA_TOKEN_LIFETIME, structs::tokens::tokens::{AccessTokenResponse, TokenEncoder}}, AppState};
use diesel::{prelude::*, result::DatabaseErrorKind};


use super::shared::{AuthCallback, Service, TempUserdataPayload};

use crate::schema::user_data::dsl::*;






#[derive(Serialize, Deserialize)]
pub struct AuthResponse{
    temp_token: String
}

pub async fn auth_callback(
    State(state): State<AppState>,
    Query(params): Query<AuthCallback>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let provider = state.tokens.get_crfs(&params.state)?.ok_or(StatusCode::UNAUTHORIZED)?;
    let user_info : UserInfo  = match provider.as_str() {
        GOOGLE_PROVIDER => google_callback(state.google_client, params.code).await?.into(),
        DISCORD_PROVIDER => discord_callback(state.discord_client, params.code).await?.into(),
        _ => {return Err(StatusCode::UNAUTHORIZED)}
    };


    let service_id = user_info.id.clone();
    let user : Option<UserData> = match &user_info.service {
        Service::Discord => {
            state.postgre.interact(move |conn|{
                user_data.select(UserData::as_select()).filter(discord_id.eq(&service_id)).load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
            }).await?
        }
        Service::Google => {
            state.postgre.interact(move |conn|{
                user_data.select(UserData::as_select()).filter(google_id.eq(&service_id)).load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
            }).await?
        }
    }.first().cloned();
    
    let tmpr_uuid = Uuid::new_v4();
    let payload = TempUserdataPayload{
        tuid: tmpr_uuid,
        user_info,
        user_uuid: user.and_then(|u| Some(u.uuid)),
        expires_at: Utc::now().timestamp() + TEMPORARY_USERDATA_TOKEN_LIFETIME as i64
    };
    let token = TokenEncoder::encode_temp(payload)?;
    //let _ : () = state.tokens.set_tmpr(tmpr_uuid)?; // todo: UNUSED? due we can trust jwt.
    Ok(Json(AuthResponse{temp_token: token})) 
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest{
    temp_token: String,
    fingerprint: String
}

fn validate_token(
    token: String
) -> Result<TempUserdataPayload, StatusCode> {
    let v = TokenEncoder::decode_temp(token)?;
    if v.expires_at >= Utc::now().timestamp() {return Err(StatusCode::UNAUTHORIZED)}
    Ok(v)
}

pub async fn oauth_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
    let token_payload = validate_token(payload.temp_token)?;
    process_tokens(jar, &state, token_payload.user_uuid.ok_or(StatusCode::UNAUTHORIZED)?, payload.fingerprint, get_user_agent(&headers)).await
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest{
    fingerprint: String,
    username: String,
    password: String,
    temp_token: String,
}

pub async fn oauth_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> Result<(CookieJar, Json<AccessTokenResponse>), StatusCode> {
    if !payload.username.is_username_valid() || !payload.password.is_password_valid() {return Err(StatusCode::UNAUTHORIZED)}
    let token_payload = validate_token(payload.temp_token)?;
    if token_payload.user_uuid.is_some() {return Err(StatusCode::FOUND)}

    let user_uuid = Uuid::new_v4();
    
    let hashed = hash(payload.password.clone(), DEFAULT_COST).map_err(adapt_error)?;

    let mut google = None;
    let mut discord = None;
    match token_payload.user_info.service {
        Service::Discord => discord = Some(token_payload.user_info.id),
        Service::Google => google = Some(token_payload.user_info.id)
    }

    let user = CreateUserData{
        uuid: user_uuid,
        username: payload.username.clone(),
        nickname: payload.username,
        password: hashed,
        email: token_payload.user_info.email,
        discord_id: discord,
        google_id: google,
        created: Utc::now().timestamp(),
    };

    let _ = state.postgre.interact(move |conn| {
        UserData::create(conn, &user).map_err(|e|{
            if let diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            })
    }).await?;

    process_tokens(jar, &state, user_uuid, payload.fingerprint, get_user_agent(&headers)).await
}















