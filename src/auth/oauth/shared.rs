use jsonwebtoken::decode;
use jsonwebtoken::encode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::env::TEMPORARY_USERDATA_TOKEN_SECRET;
use crate::shared::errors::adapt_error;
use crate::shared::replace_err::ReplaceErr;
use crate::shared::structs::tokens::tokens::TokenEncoder;

pub const CRFS_LIFETIME : u64 = 2 * 60;

pub const PROVIDER_KEY : &'static str = "provider";
pub const DISCORD_PROVIDER: &'static str = "discord";
pub const GOOGLE_PROVIDER: &'static str = "google";

#[derive(Deserialize)]
pub struct AuthCallback {
    pub code: String,
    pub state: String
}



#[derive(Serialize, Deserialize, Clone)]
pub enum Service{
    Google,
    Discord
}



#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub service: Service,
    pub id: String,
    pub avatar: String,
    pub email: String,
    pub verified: bool,
    pub name: String,
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub struct TempUserdataPayload {
    pub tuid: uuid::Uuid,
    pub user_info: UserInfo,
    pub user_uuid: Option<Uuid>,
    pub expires_at: i64,
}

impl TokenEncoder {
    pub fn encode_temp(temp_payload: TempUserdataPayload) -> Result<String, StatusCode>{
        let temp = encode(&Header::new(jsonwebtoken::Algorithm::RS256), &temp_payload, &EncodingKey::from_secret(TEMPORARY_USERDATA_TOKEN_SECRET.as_bytes())).map_err(adapt_error)?;
        Ok(temp)
    }

    pub fn decode_temp(temp_token: String) -> Result<TempUserdataPayload, StatusCode>{
        let temp = decode::<TempUserdataPayload>(temp_token.as_str(), &DecodingKey::from_secret(TEMPORARY_USERDATA_TOKEN_SECRET.as_bytes()), &Validation::new(jsonwebtoken::Algorithm::RS256)).replace_err(StatusCode::UNAUTHORIZED)?;
        Ok(temp.claims)
    }
}












