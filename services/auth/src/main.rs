use std::{sync::Arc, time::Duration};

use async_nats::jetstream::Context;
use axum::{
    body::Body, error_handling::HandleErrorLayer, extract::Request, http::StatusCode, response::Response, routing::{get, post}, Router
};
use endpoints::{login::login, logout_other::logout_other, recovery_password::{recovery_password, request_password_recovery}, refresh::refresh_tokens, register::{get_criteria, register, request_register_code}, set_refresh_rules::set_refresh_rules, username::check_username};
use message_broker::publisher::build_publisher;
use shared::env_config;
use redis_utils::redis::RedisTokens;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::error;


mod endpoints;
mod repository;


#[derive(Clone)]
pub struct AppState {
    pub db : sea_orm::DatabaseConnection, // arc doesn't needed https://github.com/SeaQL/sea-orm/blob/3203a6c7ef4f737ed4ab5ee0491cf3c45d9cd71e/examples/axum_example/api/src/lib.rs#L42-L63
    pub redis_tokens: RedisTokens, // also arc
    pub publisher: Arc<Context>
}

use anyhow::Result;

env_config!(
    ".env" => ENV = Env {
        SERVICE_AUTH_PORT : u16,
        DATABASE_URL : String,
        TURNSTILE_SECRET : String,
        EMAIL_SEND_NATS_EVENT : String,
    }
    ".cfg" => CFG = Cfg {
        REFRESH_TOKEN_LIFETIME : u64 = 30 * 24 * 60 * 60, // 30 days
        ACCESS_TOKEN_LIFETIME : u64 = 15 * 60, // 15 min
        REDIS_MAX_LIVE_SESSIONS : usize = 5,
        MIN_LOGIN_LENGTH : usize,
        MAX_LOGIN_LENGTH : usize,
        MIN_PASSWORD_LENGTH : usize,
        MAX_PASSWORD_LENGTH : usize,
        MIN_NICKNAME_LENGTH : usize,
        MAX_NICKNAME_LENGTH : usize,
        RECOVERY_EMAIL_LIFETIME : u64 = 5 * 60,
        REGISTER_EMAIL_LIFETIME : u64 = 5 * 60,
        RECOVERY_TOKEN_LEN : usize = 128,
    }
);

#[tokio::main]
async fn main() -> Result<()> {
    let mut service = service::Service::begin();

    let state = AppState{
        db: db::open_database_connection().await?,
        redis_tokens: RedisTokens::default().await,
        publisher: Arc::new(build_publisher().await?)
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
    
    
    // TODO!: REQUEST LIMITER IN NGINX
    service.route(
        Router::new()
        .route("/refresh_tokens", post(refresh_tokens))
        .route("/set_refresh_rules", post(set_refresh_rules))
        .route("/get_register_criteria", get(get_criteria))
        .route("/logout_other", post(logout_other))
        .route("/check_username", get(check_username))
        .route("/request_register_code", post(request_register_code))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/recovery_password", post(recovery_password))
        .route("/request_password_recovery", post(request_password_recovery))
        .with_state(state)
        .layer(default_layer)
    );
    service.run(ENV.SERVICE_AUTH_PORT).await?;
    Ok(())
}