use std::sync::Arc;

use axum::{body::Body, http::{Response, StatusCode}, response::IntoResponse};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use regex::Regex;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Uuid, Surreal};

use crate::{ACCESS_TOKEN_LIFETIME, ACCESS_TOKEN_SECRET, CONTACT_ADMIN_MESSAGE, REFRESH_TOKEN_LIFETIME, REFRESH_TOKEN_SECRET};


#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPayload {
    pub user: String,
    pub exp: u64
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Tokens {
    pub refresh: String,
    pub access: String
}

impl Tokens {
    pub fn get_pair(username: String) -> Result<Self, jsonwebtoken::errors::Error>{
        let now = Utc::now().timestamp() as u64;
        let refresh = TokenPayload{
            user: username.clone(),
            exp: now + REFRESH_TOKEN_LIFETIME
        };
        let access = TokenPayload{
            user: username,
            exp: now + ACCESS_TOKEN_LIFETIME
        };
        let refresh = match encode(&Header::default(), &refresh, &EncodingKey::from_secret(REFRESH_TOKEN_SECRET)) {
            Ok(token) => token,
            Err(e) => return Err(e)
        };
        let access = match encode(&Header::default(), &access, &EncodingKey::from_secret(ACCESS_TOKEN_SECRET)) {
            Ok(token) => token,
            Err(e) => return Err(e)
        };
        Ok(Self{refresh, access})
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RefreshTokenRecord {
    pub refresh_token: String,
    pub uuid: String,
    pub expires: u64,
    pub username: String,
}



pub async fn create_token_record(db: &Arc<Surreal<Client>>, username: String, refresh_token: String) -> Result<(), Response<Body>>{
    let uuid = Uuid::new_v4().to_string();
    let Ok(_) = db.create::<Option<RefreshTokenRecord>>(("refresh_tokens", uuid.clone())).content(
        RefreshTokenRecord{
            uuid,
            refresh_token,
            expires: Utc::now().timestamp() as u64 + REFRESH_TOKEN_LIFETIME,
            username
        }
    ).await else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant save data to db! {}", CONTACT_ADMIN_MESSAGE)).into_response())
    };
    Ok(())
}


pub trait UsernameValidation {
    fn is_username_valid(&self) -> bool;
}

impl UsernameValidation for String {
    fn is_username_valid(&self) -> bool {
        let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        re.is_match(self)
    }
}




#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub email: String,
    pub created: u64
}

