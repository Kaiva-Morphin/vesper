use std::{backtrace::{Backtrace, BacktraceStatus}, panic::catch_unwind, time::Duration};

use axum::{
    body::Body, error_handling::HandleErrorLayer, extract::{Request, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use endpoints::{login::login, register::{self, get_criteria, register}, username::check_username};
use migration::MigratorTrait;
use sea_orm::ConnectOptions;
use serde::{Deserialize, Serialize};
use shared::{env_config, tokens::redis::RedisTokens};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};
use tracing::{error, info, warn};

use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;

mod endpoints;
mod repository;


#[derive(Clone)]
pub struct AppState {
    pub db : sea_orm::DatabaseConnection,
    pub tokens: RedisTokens,
}

use anyhow::Result;

env_config!(
    "shared.env" => ENV = Env {
        SERVICE_AUTH_PORT : u16,
        DATABASE_URL : String,
        TURNSTILE_SECRET : String,
    }
    ".cfg" => CFG = EnvCfg {
        REDIS_REFRESH_TOKEN_LIFETIME : u64 = 30 * 24 * 60 * 60, // 30 days
        REDIS_ACCESS_TOKEN_LIFETIME : u64 = 15 * 60, // 15 min
        REDIS_MAX_LIVE_SESSIONS : usize = 5,
        MIN_LOGIN_LENGTH : usize = 4,
        MAX_LOGIN_LENGTH : usize = 24,
        MIN_PASSWORD_LENGTH : usize = 4,
        MAX_PASSWORD_LENGTH : usize = 24,
        MIN_NICKNAME_LENGTH : usize = 1,
        MAX_NICKNAME_LENGTH : usize = 32,
    }
);

async fn handler(req: Request<Body>) -> impl IntoResponse {
    if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        let ip = forwarded_for.to_str().unwrap_or("unknown");
        println!("Client IP from X-Forwarded-For: {}", ip);
    }
    "Request handled"
}


#[tokio::main]
async fn main() -> Result<()>{
    shared::utils::logger::init_logger();

    let mut options = ConnectOptions::new(&ENV.DATABASE_URL);
    
    options
        .sqlx_logging(false);

    let conn = sea_orm::Database::connect(options)
        .await
        .expect("Database connection failed");

    migration::Migrator::up(&conn, None).await.unwrap();

    let state = AppState{
        db: conn,
        tokens: RedisTokens::default()
    };

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: axum::BoxError| async {
            error!("Timeout reached!");
            StatusCode::REQUEST_TIMEOUT
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(25)));

    let tracing_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(shared::layers::logging::unique_span))
        .layer(axum::middleware::from_fn(shared::layers::logging::logging_middleware))
        .layer(CatchPanicLayer::new())
        .layer(timeout_layer);

    let limit_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_too_many_requests))
        .layer(BufferLayer::new(1024))
        .layer(RateLimitLayer::new(1, Duration::from_secs(1))); // TODO!: check header for CF-Connecting-IP from cloudflare. Also limit authed users to ~10 actions/sec

    
    
    let app = Router::new()
        .route("/", get(handler))
        .route("/get_register_criteria", get(get_criteria))
        .route("/check_username", get(check_username))
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state)
        .layer(limit_layer)
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

async fn handle_too_many_requests(err: axum::BoxError) -> (StatusCode, String) {
    info!("To many requests");
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("To many requests: {}", err)
    )
}
