use std::time::Duration;

use axum::{error_handling::HandleErrorLayer, http::status::StatusCode, Router};
use shared::env_config;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::error;

use anyhow::Result;

mod endpoints;

#[derive(Clone)]
pub struct AppState {

}

env_config!(
    ".env" => ENV = Env {
        SERVICE_ROOT_PORT : u16
    }
);

#[tokio::main]
async fn main() -> Result<()>{
    let mut service = service::Service::begin();

    let state = AppState{
        // db: db::open_database_connection().await?,
        // redis_tokens: RedisConn::build(),
        // publisher: build_publisher().await?
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
    
    service.route(
        Router::new()
        .with_state(state)
        .layer(default_layer)
    );
    service.run(ENV.SERVICE_ROOT_PORT).await?;
    Ok(())
}