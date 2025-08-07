use std::sync::Arc;

use async_nats::jetstream::Context;
use ::shared::{env_config, utils::logger::init_logger};
use anyhow::Result;

mod service;
mod db;
mod proto;

use rustperms_nodes::ENV;

use crate::service::master::*;
use crate::{db::SqlStore, proto::rustperms_master_proto_server::RustpermsMasterProtoServer};

// env_config!(
//     ".env" => ENV = Env {
//         NATS_URL : String,
//         NATS_PORT : String,
//         PERM_WRITE_NATS_EVENT : String,
//         DATABASE_URL : String,
//         RUSTPERMS_MASTER_PORT: u16,
//     }
// );

// TODO: INACTIVE MASTER REPLICA FOR REPLACEMENT


pub async fn build_publisher() -> Result<Context> {
    let nats_url = format!("nats://{}:{}", ENV.NATS_URL, ENV.NATS_PORT);
    let client = async_nats::connect(nats_url).await?;
    Ok(async_nats::jetstream::new(client))
}


#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    tracing::info!("Connecting to pg...");

    let addr = format!("[::1]:{}", ENV.RUSTPERMS_MASTER_PORT).parse()?;
    // connect to bd
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.init_schema().await?;

    tracing::info!("Connecting to nats...");
    let nats_publisher = Arc::new(build_publisher().await?);
    let nats_event = ENV.PERM_WRITE_NATS_EVENT.clone();
    // fetch state
    let manager = storage.load_manager().await?;

    tracing::info!("Starting master node!");
    // start grpc listener
    tonic::transport::Server::builder()
        .add_service(RustpermsMasterProtoServer::new(MasterNode{manager, storage, nats_publisher, nats_event}))
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