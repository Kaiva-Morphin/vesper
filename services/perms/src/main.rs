use std::time::Duration;

use axum::{error_handling::HandleErrorLayer, http::status::StatusCode, middleware::from_extractor, routing::{delete, get, post, put}, Router};

use endpoints::*;
use sea_orm::DatabaseConnection;
use shared::env_config;
use layers::{auth::AuthAccessLayer, layer_with_unique_span};
use casbin_perm_lib::{middleware::{ExtractPath, PermissionAccessLayer as PAL}};

use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::{error, info, warn};

use anyhow::Result;

mod endpoints;

#[derive(Clone)]
pub struct AppState {
    db : DatabaseConnection,
}

env_config!(
    ".env" => ENV = Env {
        SERVICE_PERMS_PORT : u16,
        DATABASE_URL : String
    }
);

#[tokio::main]
async fn main() -> Result<()>{
    let mut service = service::Service::begin();

    let permissionDB = casbin_perm_lib::PermissionDB::init(&ENV.DATABASE_URL, 16).await?;
    
    let state = AppState{
        db: db::open_database_connection().await?,
        // redis_perms: RedisPerms::default().await,
    };
    // let perm_builder = PermissionAccessLayerBuilder::new(&permissionDB);

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: axum::BoxError| async {
            error!("Timeout reached!");
            StatusCode::REQUEST_TIMEOUT
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(25)));

    let default_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(layer_with_unique_span!("request ")))
        .layer(axum::middleware::from_fn(layers::logging::logging_middleware))
        .layer(CatchPanicLayer::new())
        .layer(timeout_layer);
    
    service.route(
        Router::new()
        .route("/test", get(endpoints::test::get))
        .route("/test_authed", get(endpoints::test::get_authed)
            .layer(ServiceBuilder::new().layer(AuthAccessLayer {}))
        )
        .route("/test_perm", get(endpoints::test::get_perm)
            .layer(ServiceBuilder::new().layer(AuthAccessLayer {}).layer(PAL::new("vesper.groups.test".to_string(), &state.db).await?))
        )
        .route("/test_perm/{id}", get(endpoints::test::get_perm)
            .layer(ServiceBuilder::new().layer(AuthAccessLayer {}).layer(PAL::new("vesper.groups.{id}".to_string(), &state.db).await?))
        )
        .route("/groups/{id}", get(groups::get).layer(PAL::new("vesper.groups.{id}.get".to_string(), &state.db).await?))
        .route("/groups/{id}", post(groups::post).layer(PAL::new("vesper.groups.{id}.post".to_string(), &state.db).await?))
        .route("/groups/{id}/put/{additional}", put(groups::put).layer(PAL::new("vesper.groups.{additional}.put.{id}".to_string(), &state.db).await?))
        .route("/groups/{id}", delete(groups::delete).layer(PAL::new("vesper.groups.{id}.delete".to_string(), &state.db).await?.hidden()))
        .layer(ServiceBuilder::new()
            .layer(AuthAccessLayer {})
            .layer(from_extractor::<ExtractPath>()))
        .with_state(state)
        .layer(default_layer)
    );
    service.run(ENV.SERVICE_PERMS_PORT).await?;
    Ok(())
}