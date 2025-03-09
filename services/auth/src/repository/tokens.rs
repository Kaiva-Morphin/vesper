use axum::{http::HeaderMap, Json};
use axum_extra::extract::CookieJar;
use sea_orm::{prelude::Uuid, sqlx::types::chrono::Utc};
use sha2::Digest;
use shared::tokens::{jwt::{AccessTokenPayload, AccessTokenResponse, RefreshTokenPayload, RefreshTokenRecord, TokenEncoder}, redis::RedisConn};

use anyhow::Result;
use tracing::info;

use crate::{repository::cookies::TokenCookie, AppState, CFG};


fn hash_fingerprint(fp: &String) -> String {
    format!("{:x}", sha2::Sha256::digest(fp.as_bytes()))
}

pub fn generate_access(user_id: Uuid) ->  Result<AccessTokenResponse> {
    let exp = Utc::now().timestamp() + CFG.REDIS_ACCESS_TOKEN_LIFETIME as i64;
    let access_payload = AccessTokenPayload {
        user: user_id,
        exp
    };
    let access_token = TokenEncoder::encode_access(access_payload)?;
    Ok(AccessTokenResponse{
        access_token,
        exp
    })
}



pub fn generate_and_put_refresh(
    jar: CookieJar,
    state: &AppState,
    user_id: &Uuid,
    fingerprint: String,
    user_agent: String,
    user_ip: String,
) -> Result<CookieJar> {
    let rtid: Uuid = Uuid::new_v4();
    info!("Generating refresh token for {}. ip: {} fingerprint: {} user-agent: {}", user_id, user_ip, hash_fingerprint(&fingerprint), user_agent);
    let refresh_record = RefreshTokenRecord{
        rtid,
        user: *user_id,
        fingerprint,
        user_agent,
        ip: user_ip
    };
    let refresh_payload = RefreshTokenPayload{
        rtid,
        user: *user_id,
        exp: Utc::now().timestamp() + CFG.REDIS_REFRESH_TOKEN_LIFETIME as i64
    };
    let refresh_token = TokenEncoder::encode_refresh(refresh_payload)?;
    state.redis.set_refresh(refresh_record)?;
    Ok(jar.put_refresh(refresh_token))
}

