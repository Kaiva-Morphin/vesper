use std::{backtrace::{Backtrace, BacktraceStatus}, panic::catch_unwind, time::Duration};

use async_nats::jetstream::Context;
use axum::{
    body::Body, error_handling::HandleErrorLayer, extract::{Request, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use endpoints::{login::login, logout_other::logout_other, recovery_password::{recovery_password, request_password_recovery}, refresh::refresh_tokens, register::{self, get_criteria, register, request_register_code}, set_refresh_rules::set_refresh_rules, username::check_username};
use message_broker::publisher::build_publisher;
use shared::{env_config, tokens::redis::RedisConn};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};
use tracing::{error, info, warn};

use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;

mod endpoints;
mod repository;


#[derive(Clone)]
pub struct AppState {
    pub db : sea_orm::DatabaseConnection, // arc
    pub redis: RedisConn, // arc
    pub publisher: Context
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
        MIN_LOGIN_LENGTH : usize = 4,
        MAX_LOGIN_LENGTH : usize = 24,
        MIN_PASSWORD_LENGTH : usize = 4,
        MAX_PASSWORD_LENGTH : usize = 24,
        MIN_NICKNAME_LENGTH : usize = 1,
        MAX_NICKNAME_LENGTH : usize = 32,
        RECOVERY_EMAIL_LIFETIME : u64 = 5 * 60,
        REGISTER_EMAIL_LIFETIME : u64 = 5 * 60,
        RECOVERY_TOKEN_LEN : usize = 128,
    }
);

#[tokio::main]
async fn main() -> Result<()>{
    shared::utils::logger::init_logger();


    let state = AppState{
        db: db::open_database_connection().await?,
        redis: RedisConn::default(),
        publisher: build_publisher().await?
    };

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: axum::BoxError| async {
            error!("Timeout reached!");
            StatusCode::REQUEST_TIMEOUT
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(25)));

    let tracing_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(shared::with_unique_span_layer!("request ")))
        .layer(axum::middleware::from_fn(shared::layers::logging::logging_middleware))
        .layer(CatchPanicLayer::new())
        .layer(timeout_layer);
    
    
    // TODO!: USER LIMITER IN NGINX
    let app = Router::new()
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
        .layer(tracing_layer);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", ENV.SERVICE_AUTH_PORT)).await?;

    let v = listener.local_addr();
    if let Ok(a) = v {
        info!("Listening on {}", a);
    } else {
        warn!("Failed to get local address");
    }

    axum::serve(listener, app.into_make_service()).await.unwrap();
    Ok(())
}