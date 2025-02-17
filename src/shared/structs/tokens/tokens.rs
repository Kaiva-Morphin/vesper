use axum::{response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::{encode, errors::Error, EncodingKey, Header};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{env::{ACCESS_TOKEN_SECRET, REFRESH_TOKEN_SECRET, TEMPORARY_USERDATA_TOKEN_SECRET}, errors::{adapt_error, AsStatusCode}, settings::{ACCESS_TOKEN_LIFETIME, REFRESH_TOKEN_LIFETIME}};

use super::cookies::TokenCookie;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    pub user: Uuid,
    pub created: i64,
    pub lifetime: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    pub rtid: Uuid,
    pub user: Uuid,
    pub expires_at: u64
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRecord {
    pub rtid: Uuid,
    pub user: Uuid,
    pub fingerprint: String, // todo: does it really important?
    pub ip: String,
    pub user_agent: String // todo: does it really important?
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TokenEncoder;

impl AsStatusCode for Error {
    fn as_interaction_error(&self) -> reqwest::StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_at: i64
}



impl TokenEncoder {
    pub fn encode_access(access_payload: AccessTokenPayload) -> Result<String, StatusCode>{
        let access = encode(&Header::new(jsonwebtoken::Algorithm::RS256), &access_payload, &EncodingKey::from_secret(ACCESS_TOKEN_SECRET.as_bytes())).map_err(adapt_error)?;
        Ok(access)
    }
}




