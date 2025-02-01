use chrono::Utc;
use jsonwebtoken::{encode, errors::Error, EncodingKey, Header};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{env::{ACCESS_TOKEN_SECRET, REFRESH_TOKEN_SECRET}, errors::{adapt_error, AsStatusCode}, settings::{ACCESS_TOKEN_LIFETIME, REFRESH_TOKEN_LIFETIME}};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    pub user: Uuid,
    pub expires: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    pub rtid: Uuid,
    pub user: Uuid,
    pub expires: u64
}



#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub refresh: String,
    pub access: String
}

impl AsStatusCode for Error {
    fn as_interaction_error(&self) -> reqwest::StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl TokenPair {
    pub fn generate_pair(uuid: Uuid, rtid: Uuid) -> Result<Self, StatusCode>{
        let now = Utc::now().timestamp() as u64;
        let refresh = RefreshTokenPayload{
            user: uuid.clone(),
            rtid: rtid,
            expires: now + REFRESH_TOKEN_LIFETIME
        };
        let access = AccessTokenPayload{
            user: uuid,
            expires: now + ACCESS_TOKEN_LIFETIME
        };
        let refresh = encode(&Header::default(), &refresh, &EncodingKey::from_secret(REFRESH_TOKEN_SECRET.as_bytes())).map_err(adapt_error)?;
        let access = encode(&Header::default(), &access, &EncodingKey::from_secret(ACCESS_TOKEN_SECRET.as_bytes())).map_err(adapt_error)?;
        Ok(Self{refresh, access})
    }
}



