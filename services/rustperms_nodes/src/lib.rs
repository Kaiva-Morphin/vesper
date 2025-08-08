use crate::proto::{rustperms_master_proto_client::RustpermsMasterProtoClient, rustperms_replica_proto_client::RustpermsReplicaProtoClient};

pub mod service;
pub mod db;
pub mod proto;


shared::env_config!(
    ".env" => pub ENV = Env {
        RUSTPERMS_MASTER_ADDR: String,
        RUSTPERMS_MASTER_PORT: u16,
        RUSTPERMS_REPLICA_ADDR: String,
        RUSTPERMS_REPLICA_PORT: u16,
        PERM_WRITE_NATS_EVENT : String,
        NATS_URL : String,
        NATS_PORT : String,
        DATABASE_URL : String,
});


pub async fn connect_master() -> anyhow::Result<RustpermsMasterProtoClient<tonic::transport::Channel>> {
    let addr = format!("http://{}:{}", ENV.RUSTPERMS_MASTER_ADDR, ENV.RUSTPERMS_MASTER_PORT);
    Ok(RustpermsMasterProtoClient::connect(addr).await?)
}

pub async fn connect_replica() -> anyhow::Result<RustpermsReplicaProtoClient<tonic::transport::Channel>> {
    let addr = format!("http://{}:{}", ENV.RUSTPERMS_REPLICA_ADDR, ENV.RUSTPERMS_REPLICA_PORT);
    Ok(RustpermsReplicaProtoClient::connect(addr).await?)
}