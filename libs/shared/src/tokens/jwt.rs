use axum::{body::Body, response::IntoResponse, Json};
//use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::{decode, encode, errors::Error, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
//use crate::shared::{env::{ACCESS_TOKEN_SECRET, REFRESH_TOKEN_SECRET, TEMPORARY_USERDATA_TOKEN_SECRET}, errors::{adapt_error, AsStatusCode}, settings::{ACCESS_TOKEN_LIFETIME, REFRESH_TOKEN_LIFETIME}};

//use super::cookies::TokenCookie;

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

#[derive(Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub exp: i64
}

impl IntoResponse for AccessTokenResponse {
    fn into_response(self) -> axum::response::Response {
        let json = serde_json::to_string(&self).unwrap();
        axum::response::Response::new(Body::from(json))
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub refresh_token: String,
    pub exp: i64
}

static PUBLIC_DECODING_KEY : Lazy<DecodingKey> = Lazy::new(|| {
    DecodingKey::from_rsa_pem(&include_bytes!("public.pem")).expect("Can't load public.pem")
});

static PRIVATE_ENCODING_KEY : Lazy<EncodingKey> = Lazy::new(|| {
    EncodingKey::from_rsa_pem(&include_bytes!("private.pem")).expect("Can't load private.pem")
});

static ALGORITHM : jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::RS256;

impl TokenEncoder {
    pub fn encode_access(payload: AccessTokenPayload) -> Result<String>{
        let encoded = encode(&Header::new(ALGORITHM), &payload, &PRIVATE_ENCODING_KEY)?;
        Ok(encoded)
    }

    pub fn encode_refresh(payload: RefreshTokenPayload) -> Result<String>{
        let encoded = encode(&Header::new(ALGORITHM), &payload, &PRIVATE_ENCODING_KEY)?;
        Ok(encoded)
    }

    pub fn decode_refresh(token: String) -> Option<RefreshTokenPayload> {
        let token = decode::<RefreshTokenPayload>(&token, &PUBLIC_DECODING_KEY, &Validation::new(ALGORITHM)).ok()?;
        Some(token.claims)
    }

    pub fn decode_access(token: String) -> Option<AccessTokenPayload> {
        let token = decode::<AccessTokenPayload>(&token, &PUBLIC_DECODING_KEY, &Validation::new(ALGORITHM)).ok()?;
        Some(token.claims)
    }
}



