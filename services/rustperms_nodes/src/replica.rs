use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream;
use rustperms::prelude::{AsyncManager, PermPath, PermissionPath, RustpermsDelta, RustpermsOperation};
use ::shared::{env_config, utils::logger::init_logger};

mod db;
mod service;
mod proto;

use anyhow::Result;
use sqlx::types::Uuid;

use crate::proto::rustperms_proto::rustperms_replica_proto_server::RustpermsReplicaProtoServer;
use crate::proto::rustperms_proto::{rustperms_master_proto_client::RustpermsMasterProtoClient, rustperms_replica_proto_client::RustpermsReplicaProtoClient, SnapshotResponse};
use crate::db::SqlStore;
use crate::service::replica::{start_nats_event_listener, ReplicaNode};


env_config!(
    ".env" => ENV = Env {
        NATS_URL : String,
        NATS_PORT : String,
        PERM_WRITE_NATS_EVENT : String,
        DATABASE_URL : String,

        RUSTPERMS_MASTER_PORT: u16 = 3000,
        RUSTPERMS_MASTER_ADDR: String = "[::1]".to_string(),
        RUSTPERMS_REPLICA_PORT: u16 = 3001,
        RUSTPERMS_REPLICA_ADDR: String = "[::1]".to_string(),
    }
);


async fn try_get_manager_from_replica() -> Result<AsyncManager> {
    tracing::info!("Trying to get manager from replica!");
    let mut replica_conn = RustpermsReplicaProtoClient
        ::connect(format!("http://{}:{}", ENV.RUSTPERMS_REPLICA_ADDR, ENV.RUSTPERMS_REPLICA_PORT)).await
        .inspect_err(|e|tracing::warn!("Can't establish connection with another replica, am i first?: {e}"))?;
    let SnapshotResponse{serialized_users, serialized_groups} = replica_conn
        .get_snapshot(()).await
        .inspect_err(|e| tracing::warn!("Can't request serialized manager from another replica!: {e}"))?.into_inner();
    Ok(AsyncManager
        ::from_serialized_string(&serialized_users, &serialized_groups)
        .inspect_err(|e|tracing::error!("Can't deserialize data to manager!: {e}"))?
    )
}

async fn try_get_manager_from_master() -> Result<AsyncManager> {
    tracing::info!("Trying to get manager from master!");
    let mut replica_conn = RustpermsMasterProtoClient
        ::connect(format!("http://{}:{}", ENV.RUSTPERMS_MASTER_ADDR, ENV.RUSTPERMS_MASTER_PORT)).await
        .inspect_err(|e|tracing::error!("Can't establish connection with master!: {e}"))?;
    let SnapshotResponse{serialized_users, serialized_groups} = replica_conn
        .get_snapshot(()).await
        .inspect_err(|e| tracing::error!("Can't request serialized manager from master!: {e}"))?.into_inner();
    Ok(AsyncManager
        ::from_serialized_string(&serialized_users, &serialized_groups)
        .inspect_err(|e|tracing::error!("Can't deserialize data to manager!: {e}"))?
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let addr = format!("[::1]:{}", ENV.RUSTPERMS_REPLICA_PORT).parse()?;

    let manager = 'a: {
        if let Ok(m) = try_get_manager_from_replica().await {break 'a m};
        if let Ok(m) = try_get_manager_from_master().await {break 'a m};
        let storage = db::PostgreStorage::single_connection(&ENV.DATABASE_URL).await?;
        tracing::warn!("Can't get state from nodes, getting from db instead...");
        let manager = storage.load_manager().await?;
        storage.drop().await;
        manager
    };
    let manager = Arc::new(manager);

    tracing::info!("Manager loaded!");

    // start nats event listener
    

    let manager_clone = manager.clone();
    tokio::spawn(async move {
        let result = start_nats_event_listener(manager_clone).await;
        if let Err(e) = result {
            tracing::error!("NATS consumer failed: {e}");
            std::process::exit(1);
        }
    });

    // start grpc listener
    tonic::transport::Server::builder()
        .add_service(RustpermsReplicaProtoServer::new(ReplicaNode{manager}))
        .serve(addr)
        .await?;
    Ok(())
}
