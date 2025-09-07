use anyhow::Result;
use axum::{body::Body, extract::{Multipart, State}, response::{IntoResponse, Response}, Extension, Json};
use layers::rustperms::{ExtractPath, ExtractedPathKV};
use mime::Mime;
use minio::s3::{response::PutObjectResponse, types::S3Api};
use redis_utils::users::RedisUsers;
use reqwest::StatusCode;
use shared::{tokens::jwt::AccessTokenPayload, utils::app_err::AppErr, uuid};
use tonic::IntoStreamingRequest;
use tracing::info;

use crate::{state::{validate_catbox_url, AppState, FileType}, CFG, ENV};
use futures_util::{StreamExt, TryStreamExt};
use shared::utils::app_err::ToResponseBody;
use bytes::Bytes;


macro_rules! make_upload_handlers {
    (
        $(
            $(#[$meta:meta])*
            fn $fn_name:ident, $url_fn_name:ident;
            field = $field_fn:ident;
            file_prefix = $file_prefix:expr;
            limit = $limit:expr;
            filetype = $filetype:expr;
        )*
    ) => {
        $(
            $(#[$meta])*
            pub async fn $fn_name(
                State(state): State<AppState>,
                Extension(payload): Extension<AccessTokenPayload>,
                multipart: Multipart,
            ) -> Result<impl IntoResponse, Response<Body>> {
                let url = state.upload_file(
                    multipart,
                    format!("{}_{}", $file_prefix, payload.user.simple()),
                    $filetype,
                    $limit,
                ).await?;
                info!("Uploaded {}", url);
                state.$field_fn(payload.user, url).await?;
                Ok(().into_response())
            }

            pub async fn $url_fn_name(
                State(state): State<AppState>,
                Extension(payload): Extension<AccessTokenPayload>,
                Json(url): Json<String>,
            ) -> Result<impl IntoResponse, Response<Body>> {
                if validate_catbox_url(&url).is_none() {
                    return Err(StatusCode::BAD_REQUEST.into_response());
                }
                state.$field_fn(payload.user, url).await?;
                Ok(().into_response())
            }
        )*
    }
}


// todo: proxy and more providers
make_upload_handlers! {
    fn set_profile_background, set_profile_background_url;
    field = set_bg;
    file_prefix = "bg";
    limit = CFG.MAX_PROFILE_BG_MB;
    filetype = FileType::Media;

    fn set_miniprofile_background, set_miniprofile_background_url;
    field = set_miniprofile_bg;
    file_prefix = "mini_bg";
    limit = CFG.MAX_PROFILE_BG_MB;
    filetype = FileType::Media;

    fn set_avatar, set_avatar_url;
    field = set_avatar;
    file_prefix = "avatar";
    limit = CFG.MAX_MEDIA_MB;
    filetype = FileType::Image;
}

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(path): Extension<ExtractedPathKV>,
) -> Result<impl IntoResponse, Response<Body>> {
    let Some(guid) = path.0.get("guid") else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let user_guid = uuid::Uuid::parse_str(guid).map_err(|_| guid.clone());
    let p = state.get_profile_by_user(user_guid).await?;
    Ok(Json(p).into_response())
}
pub async fn get_miniprofile(
    State(state): State<AppState>,
    Extension(path): Extension<ExtractedPathKV>,
) -> Result<impl IntoResponse, Response<Body>> {
    let Some(guid) = path.0.get("guid") else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let user_guid = uuid::Uuid::parse_str(guid).map_err(|_| guid.clone());
    match &user_guid {
        Ok(u) => info!("MiniProfile for uuid {}", u),
        Err(e) => info!("MiniProfile for string {}", e)
    }
    let p = state.get_miniprofile_by_user(user_guid).await?;
    Ok(Json(p).into_response())
}

pub async fn set_nickname(
    State(state): State<AppState>,
    Extension(payload): Extension<AccessTokenPayload>,
    Json(nickname): Json<String>,
) -> Result<impl IntoResponse, Response<Body>> {
    state.set_nickname(payload.user, nickname).await?;
    Ok(().into_response())
}

pub async fn set_miniprofile_theme(
    State(state): State<AppState>,
    Extension(payload): Extension<AccessTokenPayload>,
    Json(theme): Json<String>,
) -> Result<impl IntoResponse, Response<Body>> {
    state.set_miniprofile_theme(payload.user, theme).await?;
    Ok(().into_response())
}

pub async fn set_profile_theme(
    State(state): State<AppState>,
    Extension(payload): Extension<AccessTokenPayload>,
    Json(theme): Json<String>,
) -> Result<impl IntoResponse, Response<Body>> {
    state.set_profile_theme(payload.user, theme).await?;
    Ok(().into_response())
}

pub async fn get_all_users(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response<Body>> {
    let users = state.cache.get_user_guids().await.trough_app_err()?;
    Ok(Json(users).into_response())
}