use axum::{body::Body, extract::Multipart, http::Response, response::IntoResponse};
use mime::Mime;
use minio::s3::types::S3Api;
use postgre_entities::user_data;
use postgre_entities::user_mini_profile;
use postgre_entities::user_profile;
use rand::rand_core::le;
use redis_utils::redis::RedisConn;
use redis_utils::redis_cache::RedisCache;
use redis_utils::users::RedisUsers;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use futures_util::{StreamExt, TryStreamExt};
use bytes::Bytes;
use sea_orm::EntityTrait;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use serde::Deserialize;
use serde::Serialize;
use shared::default_err;
use shared::{utils::app_err::ToResponseBody, uuid::Uuid};
use sea_orm::Set;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use tracing::info;
use sea_orm::RelationTrait;
use crate::CFG;
use crate::ENV;









#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub store: minio::s3::Client,
    pub cache: RedisConn,
}



pub enum FileType {
    Any,
    Image,
    Video,
    Media
}

impl FileType {
    fn is_allowed_mime(&self, content_type: &str) -> bool {
        let mime: Mime = content_type.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);
        match (self, mime.type_()) {
            (FileType::Image, mime::IMAGE) => true,
            (FileType::Video, mime::VIDEO) => true,
            (FileType::Media, mime::IMAGE | mime::VIDEO) => true,
            (FileType::Any, _) => true,
            _ => false
        }
    }
    fn is_allowed_file_extension(&self, extension: &str) -> bool {
        match self {
            FileType::Any => true,
            FileType::Image => extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") || extension.eq_ignore_ascii_case("png"),
            FileType::Video => extension.eq_ignore_ascii_case("mp4") || extension.eq_ignore_ascii_case("mov") || extension.eq_ignore_ascii_case("webm"),
            FileType::Media => FileType::Image.is_allowed_file_extension(extension) || FileType::Video.is_allowed_file_extension(extension),
        }
    }
}


fn get_extension(file_name: Option<String>) -> Option<String> {
    if let Some(name) = file_name {
        if let Some(ext) = name.rsplit('.').next() {
            if ext != name {
                return Some(ext.to_lowercase());
            }
        }
    }
    None
}
use url::Url;

pub fn validate_catbox_url(raw: &str) -> Option<()> {
    info!("Validating {}", raw);
    let url = Url::parse(raw).ok()?;
    info!("Validated {}", url);
    info!("Scheme: {}", url.scheme());
    info!("Host: {}", url.host_str()?);
    if url.scheme() != "https" || url.host_str()? != "files.catbox.moe" {
        return None
    }

    let path = url.path();
    info!("Path: {}", path);
    let file_name = path.rsplit('/').next()?;
    info!("File name: {}", file_name);
    let ext = file_name.rsplit('.').next()?.to_lowercase();
    info!("Extension: {}", ext);
    let allowed_exts = ["jpg", "jpeg", "png", "webp", "gif", "mp4", "webm"];
    if allowed_exts.contains(&ext.as_str()) {
        Some(())
    } else {
        None
    }
}


const PROFILE_PREFIX : &str = "PROFILE:";
fn profile_key(user_guid: Uuid) -> String {
    format!("{}{}", PROFILE_PREFIX, user_guid.simple())
}
const MINIPROFILE_PREFIX : &str = "PROFILE:";
fn miniprofile_key(user_guid: Uuid) -> String {
    format!("{}{}", MINIPROFILE_PREFIX, user_guid.simple())
}
#[derive(Deserialize, Serialize, Clone)]
pub struct Profile {
    pub uid: String,
    pub nickname: String,
    pub encoded_theme: Option<String>,
    pub background: Option<String>,
    pub status: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MiniProfile {
    pub uid: String,
    pub nickname: String,
    pub encoded_theme: Option<String>,
    pub background: Option<String>,
    pub status: Option<String>,
    pub avatar: Option<String>,
}


impl AppState {
    pub async fn clear_profile_cache(&self, user_guid: Uuid) -> Result<(), Response<Body>> {
        self.cache.del(&profile_key(user_guid)).await.map_err(|e| e.to_string()).trough_app_err()?;
        Ok(())
    }

    pub async fn get_profile_cache_guid(&self, user_guid: Uuid) -> Result<Option<Profile>, Response<Body>> {
        self.get_profile_cache(&profile_key(user_guid)).await
    }

    pub async fn get_profile_cache(&self, user_guid: &str) -> Result<Option<Profile>, Response<Body>> {
        let p : Option<Profile> = self.cache.get(user_guid).await.trough_app_err()?;
        Ok(p)
    }
    pub async fn set_profile_cache(&self, user_guid: Uuid, profile: Profile) -> Result<(), Response<Body>> {
        self.cache.set(&profile_key(user_guid), &profile, CFG.PROFILE_CACHE_TTL).await.map_err(|e| e.to_string()).trough_app_err()?;
        Ok(())
    }

    pub async fn clear_miniprofile_cache(&self, user_guid: Uuid) -> Result<(), Response<Body>> {
        self.cache.del(&miniprofile_key(user_guid)).await.map_err(|e| e.to_string()).trough_app_err()?;
        Ok(())
    }

    pub async fn get_miniprofile_cache_guid(&self, user_guid: Uuid) -> Result<Option<MiniProfile>, Response<Body>> {
        self.get_miniprofile_cache(&miniprofile_key(user_guid)).await
    }

    pub async fn get_miniprofile_cache(&self, user_guid: &str) -> Result<Option<MiniProfile>, Response<Body>> {
        let p : Option<MiniProfile> = self.cache.get(user_guid).await.trough_app_err()?;
        Ok(p)
    }

    pub async fn set_miniprofile_cache(&self, user_guid: Uuid, profile: MiniProfile) -> Result<(), Response<Body>> {
        self.cache.set(&miniprofile_key(user_guid), &profile, CFG.MINIPROFILE_CACHE_TTL).await.map_err(|e| e.to_string()).trough_app_err()?;
        Ok(())
    }

    pub async fn set_bg(&self, user_guid: Uuid, url: String) -> Result<(), Response<Body>> {
        info!("Bg set {} for {}", url, user_guid);
        let p = user_profile::ActiveModel{
            user_guid: Set(user_guid),
            background: Set(Some(url)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_profile_cache(user_guid).await?;
        Ok(())
    }

    pub async fn set_miniprofile_bg(&self, user_guid: Uuid, url: String) -> Result<(), Response<Body>> {
        info!("Bg set {} for {}", url, user_guid);
        let p = user_mini_profile::ActiveModel{
            user_guid: Set(user_guid),
            background: Set(Some(url)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_miniprofile_cache(user_guid).await?;
        Ok(())
    }
    
    pub async fn set_profile_status(&self, user_guid: Uuid, status: String) -> Result<(), Response<Body>> {
        info!("Status set {} for {}", status, user_guid);
        let p = user_profile::ActiveModel{
            user_guid: Set(user_guid),
            status: Set(Some(status)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_profile_cache(user_guid).await?;
        Ok(())
    }

    pub async fn set_miniprofile_status(&self, user_guid: Uuid, status: String) -> Result<(), Response<Body>> {
        info!("Status set {} for {}", status, user_guid);
        let p = user_mini_profile::ActiveModel{
            user_guid: Set(user_guid),
            status: Set(Some(status)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_miniprofile_cache(user_guid).await?;
        Ok(())
    }

    pub async fn set_avatar(&self, user_guid: Uuid, url: String) -> Result<(), Response<Body>> {
        info!("Avatar set {} for {}", url, user_guid);
        let p = user_data::ActiveModel{
            guid: Set(user_guid),
            avatar: Set(Some(url)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_profile_cache(user_guid).await?;
        self.clear_miniprofile_cache(user_guid).await?;
        Ok(())
    }

    


    pub async fn get_profile_by_user(&self, uid_or_guid: Result<Uuid, String>) -> Result<Profile, Response<Body>> {
        match uid_or_guid {
            Ok(guid) => {
                if let None = self.cache.get_user_uid(&guid).await.trough_app_err()? {return Err(StatusCode::NOT_FOUND.into_response())};
                self.get_profile(guid).await
            }
            Err(uid) => {
                let Some(guid) = self.cache.get_user_guid(&uid).await.trough_app_err()? else {return Err(StatusCode::NOT_FOUND.into_response())};
                self.get_profile(guid).await
            }
        }
    }

    pub async fn get_profile(&self, user_guid: Uuid) -> Result<Profile, Response<Body>> {
        let profile = self.get_profile_cache_guid(user_guid).await?;
        if let Some(profile) = profile {
            return Ok(profile);
        }
        let p = user_profile::Entity::find()
            .filter(user_profile::Column::UserGuid.eq(user_guid))
            .join(JoinType::LeftJoin, user_profile::Relation::UserData.def())
            .select_also(user_data::Entity)
            .one(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        if let Some((p, Some(ud))) = p {
            let profile = Profile{
                uid: ud.uid,
                nickname: ud.nickname,
                encoded_theme: p.encoded_theme,
                background: p.background,
                status: p.status,
                avatar: ud.avatar,
            };
            self.set_profile_cache(user_guid, profile.clone()).await?;
            Ok(profile)
        } else {
            Err(StatusCode::NOT_FOUND.into_response())
        }
    }

    pub async fn get_miniprofile_by_user(&self, uid_or_guid: Result<Uuid, String>) -> Result<MiniProfile, Response<Body>> {
        match uid_or_guid {
            Ok(guid) => {
                info!("Checking user to exist");
                if let None = self.cache.get_user_uid(&guid).await.trough_app_err()? {return Err(StatusCode::NOT_FOUND.into_response())};
                info!("User exists");
                self.get_miniprofile(guid).await
            }
            Err(uid) => {
                let Some(guid) = self.cache.get_user_guid(&uid).await.trough_app_err()? else {return Err(StatusCode::NOT_FOUND.into_response())};
                self.get_miniprofile(guid).await
            }
        }
    }

    pub async fn get_miniprofile(&self, user_guid: Uuid) -> Result<MiniProfile, Response<Body>> {
        let profile = self.get_miniprofile_cache_guid(user_guid).await?;
        if let Some(profile) = profile {
            return Ok(profile);
        }
        let p = user_mini_profile::Entity::find()
            .filter(user_mini_profile::Column::UserGuid.eq(user_guid))
            .join(JoinType::LeftJoin, user_mini_profile::Relation::UserData.def())
            .select_also(user_data::Entity)
            .one(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        if let Some((p, Some(ud))) = p {
            let profile = MiniProfile{
                uid: ud.uid,
                nickname: ud.nickname,
                encoded_theme: p.encoded_theme,
                background: p.background,
                status: p.status,
                avatar: ud.avatar,
            };
            self.set_miniprofile_cache(user_guid, profile.clone()).await?;
            Ok(profile)
        } else {
            Err(StatusCode::NOT_FOUND.into_response())
        }
    }


    pub async fn upload_file(&self, mut multipart: Multipart, name: String, allowed: FileType, limit_mb: f32) -> Result<String, Response<Body>> {
        if let Some(field) = multipart.next_field().await.map_err(|e| e.to_string()).trough_app_err()? {
            let file_name = field.file_name().map(|s| s.to_string());
            let Some(ext) = get_extension(file_name) else {
                return Err((StatusCode::BAD_REQUEST, "Invalid file extension").into_response());
            };
            if !allowed.is_allowed_file_extension(&ext) {
                return Err((StatusCode::BAD_REQUEST, "Invalid file extension").into_response());
            }

            let content_type = field.content_type().unwrap_or("application/octet-stream");
            if !allowed.is_allowed_mime(content_type) {
                return Err((StatusCode::BAD_REQUEST, "Only media files (images/videos/gifs) are allowed").into_response());
            }
            let mut stream = field.into_stream();
            let mut buffer = Vec::new();
            let mut total_size = 0usize;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| e.to_string()).trough_app_err()?;
                total_size += chunk.len();

                if total_size > (limit_mb * 1024.0 * 1024.0) as usize {
                    return Err((StatusCode::BAD_REQUEST, format!("File too large, limit is {}MB", limit_mb)).into_response());
                }

                buffer.extend_from_slice(&chunk);
            }

            let data = Bytes::from(buffer);
            let result = self
                .store
                .put_object(ENV.MINIO_BUCKET_NAME.clone(), &format!("{}.{}", name, ext.to_lowercase()), data.into())
                .send()
                .await
                .map_err(|e| e.to_string()).trough_app_err()?;
            return Ok(format!("/api/store/{}/{}",result.bucket,result.object))
        }
        Err((StatusCode::BAD_REQUEST, "No valid file found").into_response())
    }

    pub async fn set_nickname(&self, user_guid: Uuid, nickname: String) -> Result<(), Response<Body>> {
        info!("Nickname set {} for {}", nickname, user_guid);
        let p = user_data::ActiveModel{
            guid: Set(user_guid),
            nickname: Set(nickname),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_profile_cache(user_guid).await?;
        self.clear_miniprofile_cache(user_guid).await?;

        self.clear_miniprofile_cache(user_guid).await?;
        self.clear_profile_cache(user_guid).await?;
        Ok(())
    }

    pub async fn set_miniprofile_theme(&self, user_guid: Uuid, theme: String) -> Result<(), Response<Body>> {
        info!("Theme set {} for {}", theme, user_guid);
        let p = user_mini_profile::ActiveModel{
            user_guid: Set(user_guid),
            encoded_theme: Set(Some(theme)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_miniprofile_cache(user_guid).await?;
        Ok(())
    }

    pub async fn set_profile_theme(&self, user_guid: Uuid, theme: String) -> Result<(), Response<Body>> {
        info!("Theme set {} for {}", theme, user_guid);
        let p = user_profile::ActiveModel{
            user_guid: Set(user_guid),
            encoded_theme: Set(Some(theme)),
            ..Default::default()
        };
        p.update(&self.db).await.map_err(|e| e.to_string()).trough_app_err()?;
        self.clear_profile_cache(user_guid).await?;
        Ok(())
    }
}