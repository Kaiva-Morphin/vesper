use std::time::Duration;

use axum::{error_handling::HandleErrorLayer, http::status::StatusCode, routing::get, Router};
use shared::{env_config, layers::{auth::AuthAccessLayer, permission::PermissionAccessLayer}};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::{error, info, warn};

use anyhow::Result;

mod endpoints;

#[derive(Clone)]
pub struct AppState {

}

env_config!(
    ".env" => ENV = Env {
        SERVICE_PERMS_PORT : u16
    }
);

#[tokio::main]
async fn main() -> Result<()>{
    shared::utils::logger::init_logger();

    let state = AppState{
        // db: db::open_database_connection().await?,
        // redis_tokens: RedisConn::for_tokens(),
        // publisher: build_publisher().await?
    };

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: axum::BoxError| async {
            error!("Timeout reached!");
            StatusCode::REQUEST_TIMEOUT
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(25)));

    let default_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(shared::layer_with_unique_span!("request ")))
        .layer(axum::middleware::from_fn(shared::layers::logging::logging_middleware))
        .layer(CatchPanicLayer::new())
        .layer(timeout_layer);
    
    let db = db::open_database_connection().await?;
    
    let app = Router::new()
        .route("/test", get(endpoints::test::get))
        .route("/test_authed", get(endpoints::test::get_authed)
            .layer(ServiceBuilder::new().layer(AuthAccessLayer {}))
        )
        .route("/test_perm", get(endpoints::test::get_perm)
            .layer(ServiceBuilder::new().layer(AuthAccessLayer {}).layer(PermissionAccessLayer::create_and_register("vesper.groups.test".to_string(), &db).await?))
        )
        .route("/test_perm/{id}", get(endpoints::test::get_perm)
            .layer(ServiceBuilder::new().layer(AuthAccessLayer {}).layer(PermissionAccessLayer::create_and_register("vesper.groups.test".to_string(), &db).await?))
        )
        .with_state(state)
        .layer(default_layer);
        
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", ENV.SERVICE_PERMS_PORT)).await?;

    let v = listener.local_addr();
    if let Ok(a) = v {
        info!("Listening on {}", a);
    } else {
        warn!("Failed to get local address");
    }

    axum::serve(listener, app.into_make_service()).await.unwrap();
    Ok(())
}