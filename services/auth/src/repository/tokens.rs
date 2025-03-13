use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use sea_orm::{prelude::Uuid, sqlx::types::chrono::Utc};
use shared::{tokens::jwt::{AccessTokenPayload, AccessTokenResponse, RefreshRules, RefreshTokenPayload, RefreshTokenRecord, TokenEncoder}, utils::set_encoder::encode_set_to_string};

use anyhow::Result;
use tracing::info;

use crate::{repository::cookies::TokenCookie, AppState, CFG};

pub fn generate_access(user_id: Uuid) ->  Result<AccessTokenResponse> { // todo: move to state
    let exp = Utc::now().timestamp() + CFG.ACCESS_TOKEN_LIFETIME as i64;
    let access_payload = AccessTokenPayload {
        user: user_id,
        groups: encode_set_to_string(&mut (1..=10000).collect()), // TODO!: REAL GROUPS
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
    user_info: UserInfoExt,
    email: String,
    rules: RefreshRules
) -> Result<CookieJar> {
    let rtid: Uuid = Uuid::new_v4();
    info!("Generating refresh token for {}. {}", user_id, user_info);
    let refresh_record = RefreshTokenRecord{
        rtid,
        user: *user_id,
        fingerprint: user_info.fingerprint,
        user_agent: user_info.user_agent,
        ip: user_info.ip,
        email
    };
    let refresh_payload = RefreshTokenPayload{
        rtid,
        user: *user_id,
        exp: Utc::now().timestamp() + CFG.REFRESH_TOKEN_LIFETIME as i64,
        rules
    };
    let refresh_token = TokenEncoder::encode_refresh(refresh_payload)?;
    info!("Rtid {rtid}");
    state.redis_tokens.set_refresh(refresh_record)?;
    Ok(jar.put_refresh(refresh_token))
}

