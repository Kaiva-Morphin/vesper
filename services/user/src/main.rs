use std::{sync::Arc, time::Duration};

use axum::{error_handling::HandleErrorLayer, middleware::from_extractor, routing::get, Router};
use layers::{auth::AuthAccessLayer, rustperms::{ExtractPath, PermissionMiddlewareBuilder}};
use reqwest::StatusCode;
use shared::utils::logger::init_logger;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor};
use tower_http::{catch_panic::CatchPanicLayer, cors::{Any, CorsLayer}};
use tracing::{error, info};

use crate::profile::{get_mini_profile, get_profile};


use rustperms_nodes::proto::{CheckPermRequest};




mod profile;


shared::env_config!(
    ".env" => ENV = Env {
        SERVICE_USER_PORT: u16,     
});



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    let mut service = service::Service::begin();

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
    let mut replica = rustperms_nodes::connect_replica().await?;


    let r = replica.get_snapshot(()).await?;
    tracing::info!("{r:#?}");

    let r = replica.check_perm(CheckPermRequest{user_uid: "".to_string(), permission: "dev.default.wildcard.123".to_string(), unset_policy: false}).await?;
    tracing::info!("{r:#?}");

    let p = PermissionMiddlewareBuilder::new(replica);

    service.route(
        Router::new()
            .nest("/api/user/profile/{guid}", Router::new().route("/{id}", get(get_profile)).layer(p.build("dev.default.any.{id}.dev").await?))

            .layer(AuthAccessLayer::allow_guests())
            .layer(from_extractor::<ExtractPath>())
            .layer(cors)
            .layer(default_layer)
    );
    service.run(ENV.SERVICE_USER_PORT).await?;
    Ok(())
}


            // .route("/api/user/profile/default/any/{id}", get(get_profile)).layer(p.build("dev.default.any.{id}.dev").await?)
            // .route("/api/user/profile/default/exact", get(get_profile)).layer(p.build("dev.default.exact").await?)
            // .route("/api/user/profile/guest/wildcard/{id}", get(get_profile)).layer(p.build("dev.guest.wildcard.{id}").await?)
            // .route("/api/user/profile/guest/any_wrong/{id}", get(get_profile)).layer(p.build("dev.guest.any.{id}").await?)
            // .route("/api/user/profile/guest/any/{id}", get(get_profile)).layer(p.build("dev.guest.any.{id}.dev").await?)
            // .route("/api/user/profile/guest/exact", get(get_profile)).layer(p.build("dev.guest.exact").await?)
            // .route("/api/user/profile/logged/wildcard/{id}", get(get_profile)).layer(p.build("dev.logged.wildcard.{id}").await?)
            // .route("/api/user/profile/logged/any_wrong/{id}", get(get_profile)).layer(p.build("dev.logged.any.{id}").await?)
            // .route("/api/user/profile/logged/any/{id}", get(get_profile)).layer(p.build("dev.logged.any.{id}.dev").await?)
            // .route("/api/user/profile/logged/exact", get(get_profile)).layer(p.build("dev.logged.exact").await?)