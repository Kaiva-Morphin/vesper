use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use redis_utils::redis_tokens::RedisTokens;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::TokenEncoder, utils::app_err::AppErr};
use tracing::info;

use crate::{repository::cookies::TokenCookie, AppState};

pub async fn logout(
    State(state): State<AppState>,
    mut jar: CookieJar,
    Extension(user_info) : Extension<UserInfoExt>,
) -> Result<impl IntoResponse, AppErr> {
    info!("Logging out user: {}", user_info);
    let Some(refresh_token_string) = jar.get_refresh() else {return Ok((jar.rm_refresh(), StatusCode::UNAUTHORIZED).into_response())};
    jar = jar.rm_refresh();
    let Some(refresh_payload) = TokenEncoder::decode_refresh(refresh_token_string) else {return Ok((jar, StatusCode::UNAUTHORIZED).into_response())};
    state.redis.rm_refresh(&refresh_payload.rtid).await?;
    Ok((jar, ()).into_response())
}