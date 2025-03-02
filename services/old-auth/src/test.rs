use axum::{routing::post, Json, Router};
use axum_extra::extract::CookieJar;
use cookie::{time::Duration, Cookie};
use jsonwebtoken::{decode, encode, EncodingKey, DecodingKey, Header};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    pub user: String,
    pub exp: i64
}



pub fn encode_refresh(payload: RefreshTokenPayload) -> Result<String, StatusCode>{
    let encoded = encode(&Header::new(jsonwebtoken::Algorithm::RS256), &payload,
    &EncodingKey::from_rsa_pem(include_bytes!("../private.pem")).unwrap()).unwrap();
    Ok(encoded)
}

pub fn decode_refresh(token: String) -> Result<RefreshTokenPayload, StatusCode>{
    let token = decode::<RefreshTokenPayload>(&token, &DecodingKey::from_secret("secret".as_ref()), &Validation::new(Algorithm::RS256))?;
    let encoded = encode(&Header::new(jsonwebtoken::Algorithm::RS256), &payload,
    &EncodingKey::from_rsa_pem(include_bytes!("../private.pem")).map_err(adapt_error)?).map_err(adapt_error)?;
    Ok(encoded)
}