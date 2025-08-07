use std::{sync::Arc, time::Duration};

use async_nats::jetstream::Context;
use axum::{
    error_handling::HandleErrorLayer, http::{HeaderValue, Request, StatusCode}, middleware::Next, response::IntoResponse, routing::{delete, get, post, put}, Router
};
use endpoints::{login::login, logout_other::logout_other, recovery_password::{recovery_password, request_password_recovery}, refresh::refresh_tokens, register::{register, request_register_code}, set_refresh_rules::set_refresh_rules, username::check_user_uid};
use message_broker::publisher::build_publisher;
use shared::env_config;
use redis_utils::redis::RedisTokens;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tower_http::{catch_panic::CatchPanicLayer, cors::{Any, CorsLayer}};
use tracing::error;


mod endpoints;
mod repository;

use rustperms_nodes::proto::rustperms_master_proto_client::RustpermsMasterProtoClient;

#[derive(Clone)]
pub struct AppState {
    pub db : sea_orm::DatabaseConnection, // arc doesn't needed https://github.com/SeaQL/sea-orm/blob/3203a6c7ef4f737ed4ab5ee0491cf3c45d9cd71e/examples/axum_example/api/src/lib.rs#L42-L63
    pub redis_tokens: RedisTokens, // also arc
    pub publisher: Arc<Context>,
    pub google_client: GoogleClient,
    pub rustperms_master: RustpermsMasterProtoClient<tonic::transport::Channel>,
}

use anyhow::Result;

use crate::endpoints::{delete::delete_account, oauth::{build_google_client, login_discord, login_google, oauth_callback, oauth_login, oauth_register, GoogleClient}, timestamp::get_timestamp};

env_config!(
    ".env" => ENV = Env {
        SERVICE_AUTH_PORT : u16,
        DATABASE_URL : String,
        TURNSTILE_SECRET : String,
        EMAIL_SEND_NATS_EVENT : String,

        DISCORD_AUTH_URI: String,
        
        GOOGLE_REDIRECT_URI : String,
        GOOGLE_CLIENT_SECRET : String,
        GOOGLE_CLIENT_ID : String,
    }
    ".cfg" => CFG = Cfg {
        REFRESH_TOKEN_LIFETIME : u64 = 30 * 24 * 60 * 60, // 30 days
        ACCESS_TOKEN_LIFETIME : u64 = 15 * 60, // 15 min
        REDIS_MAX_LIVE_SESSIONS : usize = 5,
        MIN_NICKNAME_LENGTH : usize,
        MAX_NICKNAME_LENGTH : usize,
        RECOVERY_EMAIL_LIFETIME : u64 = 5 * 60,
        REGISTER_EMAIL_LIFETIME : u64 = 5 * 60,
        RECOVERY_TOKEN_LEN : usize = 128,

        USERNAME_CHECKS_PER_SEC : u64 = 10,
    }
);

#[tokio::main]
async fn main() -> Result<()> {
    let mut service = service::Service::begin();
    let m = rustperms_nodes::connect_master().await?;
    let state = AppState{
        db: db::open_database_connection().await?,
        redis_tokens: RedisTokens::default().await,
        publisher: Arc::new(build_publisher().await?),
        google_client: build_google_client(),
        rustperms_master: m,
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
    
    service.route(
        Router::new()
            .route("/api/auth/account", post(register))
            .route("/api/auth/account", delete(delete_account))
            .route("/api/auth/account/uid_check", get(check_user_uid).layer(GovernorLayer {config: governor_conf,}))
            .route("/api/auth/account/request_register_code", post(request_register_code))

            .route("/api/auth/session", post(login))
            .route("/api/auth/session", delete(logout_other))

            .route("/api/auth/tokens/refresh", post(refresh_tokens))
            .route("/api/auth/tokens/rules", put(set_refresh_rules))

            .route("/api/auth/password_recovery", get(request_password_recovery))
            .route("/api/auth/password_recovery", post(recovery_password))

            .route("/api/auth/oauth", get(oauth_callback)) //.layer(axum::middleware::from_fn(add_coop))

            .route("/api/auth/google", get(login_google))
            .route("/api/auth/discord", get(login_discord))

            .route("/api/auth/oauth/account", post(oauth_register))
            .route("/api/auth/oauth/session", post(oauth_login))

            .route("/api/auth/timestamp", get(get_timestamp))

            .with_state(state)
            .layer(cors)
            .layer(default_layer)
    );
    service.run_with_connect_info(ENV.SERVICE_AUTH_PORT).await?;
    Ok(())
}


async fn add_coop(req: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    response.headers_mut().insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );
    response
}


