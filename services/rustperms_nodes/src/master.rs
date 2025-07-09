use rustperms::{api::actions::RustpermsDelta, prelude::*};
use ::shared::env_config;
use anyhow::Result;
use tonic::{body::Body, Request, Response};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::{info, Span};

use crate::{db::SqlStore, service::master::{rustperms_master::rustperms_master_proto_server::RustpermsMasterProtoServer, MasterNode}};
mod shared;
mod db;
mod service;
use service::master::*;
use tower::ServiceBuilder;

env_config!(
    ".env" => ENV = Env {
        NATS_URL : String,
        NATS_PORT : String,
        PERM_WRITE_NATS_EVENT : String,
        DATABASE_URL : String,
        RUSTPERMS_MASTER_PORT: u16 = 3000,
    }
);

// TODO: INACTIVE MASTER REPLICA FOR REPLACEMENT

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::layer::SubscriberExt::with(tracing_subscriber::registry(), tracing_subscriber::fmt::layer());
    tracing::subscriber::set_global_default(subscriber).ok();

    let addr = format!("[::1]:{}", ENV.RUSTPERMS_MASTER_PORT).parse()?;
    // // connect to bd
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    // fetch state
    storage.init_schema().await?;
    let manager = storage.load_manager().await?;


    // start grpc listener
    tonic::transport::Server::builder()
        .add_service(RustpermsMasterProtoServer::new(MasterNode{manager, storage}))
        .serve(addr)
        .await?;
    Ok(())
    
}


// pub async fn logging_middleware(mut req: Request<Body>, next: Next) -> Response {
//     let span = Span::current();
//     // info!("Received request on: {}. {}", req.uri().to_string(), user_info);
//     // req.extensions_mut().insert(user_info);
//     let response = next.run(req).instrument(span).await;
//     info!("Response status: {:?}", response.status());
//     response
// }