use std::{sync::Arc, time::Duration};

use axum::{error_handling::HandleErrorLayer, middleware::from_extractor, routing::{get, put}, Router};
use layers::{auth::AuthAccessLayer, rustperms::{ExtractPath, PermissionMiddlewareBuilder}};
use minio::s3::{creds::StaticProvider, Client};
use redis_utils::{redis::RedisConn, redis_cache::RedisCache};
use reqwest::StatusCode;
use shared::{router, utils::logger::init_logger};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor};
use tower_http::{catch_panic::CatchPanicLayer, cors::{Any, CorsLayer}};
use tracing::{error, info};

use crate::profile::*;


mod profile;
mod state;


shared::env_config!(
    ".env" => ENV = Env {
        SERVICE_USER_PORT: u16,     
        MINIO_BUCKET_NAME: String,
        MINIO_ROOT_USER: String,
        MINIO_URL: String,
        MINIO_ROOT_PASSWORD: String,
    }
    ".cfg" => CFG = Cfg {
        MAX_MEDIA_MB: f32,
        MAX_MINI_PROFILE_BG_MB: f32,
        MAX_PROFILE_BG_MB: f32,
        PROFILE_CACHE_TTL: u64,
        MINIPROFILE_CACHE_TTL: u64,
    }
);

use crate::profile::set_profile_background;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    let mut service = service::Service::begin();
    let client = Client::new(
        ENV.MINIO_URL.parse()?,
        Some(Box::new(StaticProvider::new(
            &ENV.MINIO_ROOT_USER,
            &ENV.MINIO_ROOT_PASSWORD,None))),
        None,
        None
    ).unwrap();

    let state = state::AppState{
        db: db::open_database_connection().await?,
        store: client,
        cache: RedisConn::default().await,
    };

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: axum::BoxError| async {
            error!("Timeout reached!");
            StatusCode::REQUEST_TIMEOUT
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(25)));

    let default_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(layers::layer_with_unique_span!("request ")))
        .layer(axum::middleware::from_fn(layers::logging::logging_middleware))
        .layer(CatchPanicLayer::new())
        .layer(timeout_layer);

    let interval = Duration::from_secs(60);
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            // .per_second(CFG.USERNAME_CHECKS_PER_SEC)
            // .burst_size((interval.as_secs() * CFG.USERNAME_CHECKS_PER_SEC * 2) as u32)
            .finish()
            .unwrap(),
    );
    let governor_limiter = governor_conf.limiter().clone();
    std::thread::spawn(move || {
       loop {
           std::thread::sleep(interval);
           let l = governor_limiter.len();
           if l != 0 {tracing::info!("rate limiting storage size: {l}");}
           governor_limiter.retain_recent();
       }
   });
   // TODO!: DEV ONLY
   let cors = CorsLayer::new()
        .allow_origin("http://localhost:1420".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));
    let replica = rustperms_nodes::connect_replica().await?;
    let p = PermissionMiddlewareBuilder::new(replica);


    service.route(
        router!(
            p,
            "/api/user/edit" : (AuthAccessLayer::only_authorized()) => {
                put "/profile/bg" -> set_profile_background ("user.profile.edit.{from_access}.bg")
                put "/profile/bg_url" -> set_profile_background_url ("user.profile.edit.{from_access}.bg")
                put "/profile/theme" -> set_profile_theme ("user.profile.edit.{from_access}.theme")

                put "/avatar" -> set_avatar ("user.profile.edit.{from_access}.avatar")
                put "/avatar_url" -> set_avatar_url ("user.profile.edit.{from_access}.avatar")
                put "/nickname" -> set_nickname ("user.profile.edit.{from_access}.nickname")

                put "/miniprofile/bg" -> set_miniprofile_background ("user.miniprofile.edit.{from_access}.bg")
                put "/miniprofile/bg_url" -> set_miniprofile_background_url ("user.miniprofile.edit.{from_access}.bg")
                put "/miniprofile/theme" -> set_miniprofile_theme ("user.miniprofile.edit.{from_access}.theme")
            }
            "/api/user" : (AuthAccessLayer::allow_guests()) => {
                get "/guids" -> get_all_users
                get "/profile/{guid}" -> get_profile ("user.profile.view.{guid}")
                get "/miniprofile/{guid}" -> get_miniprofile 
            }
        )
        .layer(from_extractor::<ExtractPath>())
        .layer(cors)
        .layer(default_layer)
        .with_state(state)
    );
    service.run(ENV.SERVICE_USER_PORT).await?;
    Ok(())
}