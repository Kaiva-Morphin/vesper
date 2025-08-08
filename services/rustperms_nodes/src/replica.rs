use std::sync::Arc;

use rustperms::prelude::AsyncManager;
use rustperms_nodes::proto::SnapshotResponse;
use ::shared::{utils::logger::init_logger};

use anyhow::Result;


use rustperms_nodes::proto::rustperms_replica_proto_server::RustpermsReplicaProtoServer;
use rustperms_nodes::db::SqlStore;
use rustperms_nodes::service::replica::{start_nats_event_listener, ReplicaNode};

use rustperms_nodes::{connect_master, connect_replica, ENV};

async fn try_get_manager_from_replica() -> Result<AsyncManager> {
    tracing::info!("Trying to get manager from replica!");
    let mut replica_conn = connect_replica().await
        .inspect_err(|e|tracing::warn!("Can't establish connection with another replica, am i first?: {e}"))?;
    let SnapshotResponse{serialized_users, serialized_groups} = replica_conn
        .get_snapshot(()).await
        .inspect_err(|e| tracing::warn!("Can't request serialized manager from another replica!: {e}"))?.into_inner();
    AsyncManager
        ::from_serialized_string(&serialized_users, &serialized_groups)
        .inspect_err(|e|tracing::error!("Can't deserialize data to manager!: {e}"))
}

async fn try_get_manager_from_master() -> Result<AsyncManager> {
    tracing::info!("Trying to get manager from master!");
    let mut replica_conn = connect_master().await
        .inspect_err(|e|tracing::error!("Can't establish connection with master!: {e}"))?;
    let SnapshotResponse{serialized_users, serialized_groups} = replica_conn
        .get_snapshot(()).await
        .inspect_err(|e| tracing::error!("Can't request serialized manager from master!: {e}"))?.into_inner();
    AsyncManager
        ::from_serialized_string(&serialized_users, &serialized_groups)
        .inspect_err(|e|tracing::error!("Can't deserialize data to manager!: {e}"))
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let addr = format!("[::1]:{}", ENV.RUSTPERMS_REPLICA_PORT).parse()?;

    let manager = 'a: {
        if let Ok(m) = try_get_manager_from_replica().await {break 'a m};
        if let Ok(m) = try_get_manager_from_master().await {break 'a m};
        let storage = rustperms_nodes::db::PostgreStorage::single_connection(&ENV.DATABASE_URL).await?;
        tracing::warn!("Can't get state from nodes, getting from db instead...");
        let manager = storage.load_manager().await?;
        storage.drop().await;
        manager
    };
    let manager = Arc::new(manager);

    tracing::info!("Manager loaded!");

    // start nats event listener
    let nats_url = format!("nats://{}:{}", ENV.NATS_URL, ENV.NATS_PORT);
    

    let manager_clone = manager.clone();
    tokio::spawn(async move {
        let result = start_nats_event_listener(manager_clone, nats_url, ENV.PERM_WRITE_NATS_EVENT.clone()).await;
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
