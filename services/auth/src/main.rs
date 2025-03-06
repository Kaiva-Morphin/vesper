use std::{backtrace::{Backtrace, BacktraceStatus}, panic::catch_unwind, time::Duration};

use axum::{
    body::Body, error_handling::HandleErrorLayer, extract::{Request, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, routing::get, Json, Router
};
use migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use shared::{env_config, utils::panic_hook::panic_hook};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};
use tracing::{error, info, warn};

use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;

mod endpoints;
mod repository;


#[derive(Clone)]
pub struct AppState {
    db : sea_orm::DatabaseConnection,
}

use anyhow::Result;

env_config!(
    "shared.env" => ENV = EnvConfig {
        SERVICE_AUTH_PORT : u16,
        DATABASE_URL : String,
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
    std::panic::set_hook(Box::new(panic_hook));

    let conn = sea_orm::Database::connect(&ENV.DATABASE_URL)
        .await
        .expect("Database connection failed");

    migration::Migrator::up(&conn, None).await.unwrap();

    let state = AppState{
        db: conn
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
        .layer(RateLimitLayer::new(1, Duration::from_secs(1)));

    let app = Router::new()
        .route("/", get(handler))
        .route("/check_username", get(endpoints::username::check_username).route_layer(limit_layer))
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

async fn handle_too_many_requests(err: axum::BoxError) -> (StatusCode, String) {
    info!("To many requests");
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("To many requests: {}", err)
    )
}
