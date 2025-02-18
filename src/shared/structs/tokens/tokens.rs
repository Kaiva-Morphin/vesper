use axum::{response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::{decode, encode, errors::Error, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
use crate::shared::{env::{ACCESS_TOKEN_SECRET, REFRESH_TOKEN_SECRET, TEMPORARY_USERDATA_TOKEN_SECRET}, errors::{adapt_error, AsStatusCode}, settings::{ACCESS_TOKEN_LIFETIME, REFRESH_TOKEN_LIFETIME}};

use super::cookies::TokenCookie;

use include_bytes_plus::include_bytes;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    pub user: Uuid,
    pub exp: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    pub rtid: Uuid,
    pub user: Uuid,
    pub exp: i64
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
    pub exp: i64
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub refresh_token: String,
    pub exp: i64
}

impl TokenEncoder {
    pub fn encode_access(payload: AccessTokenPayload) -> Result<String>{
        let encoded = encode(&Header::new(jsonwebtoken::Algorithm::RS256), &payload,
        &EncodingKey::from_rsa_pem(&include_bytes!("private.pem"))?)?;
        Ok(encoded)
    }


    pub fn encode_refresh(payload: RefreshTokenPayload) -> Result<String>{
        let encoded = encode(&Header::new(jsonwebtoken::Algorithm::RS256), &payload,
        &EncodingKey::from_rsa_pem(&include_bytes!("private.pem"))?)?;
        Ok(encoded)
    }

    pub fn decode_refresh(token: String) -> Result<RefreshTokenPayload> {
        let token = decode::<RefreshTokenPayload>(&token, &DecodingKey::from_rsa_pem(&include_bytes!("public.pem"))?, &Validation::new(Algorithm::RS256))?;
        Ok(token.claims)
    }

    pub fn decode_access(token: String) -> Result<AccessTokenPayload> {
        let token = decode::<AccessTokenPayload>(&token, &DecodingKey::from_rsa_pem(&include_bytes!("public.pem"))?, &Validation::new(Algorithm::RS256))?;
        Ok(token.claims)
    }
}



