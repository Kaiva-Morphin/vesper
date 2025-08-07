use migration::MigratorTrait;
use rustperms_nodes::proto::{rustperms_master_proto_client::RustpermsMasterProtoClient, WriteRequest};
use shared::utils::logger::init_logger;
use tracing::info;


shared::env_config!(
    ".env" => ENV = Env {
        RUSTPERMS_MASTER_ADDR: String,
        RUSTPERMS_MASTER_PORT: u16,
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    info!("Connecting to database...");
    let conn = db::open_database_connection().await.unwrap();
    shared::tracing::info!("Running migrations...");
    migration::Migrator::up(&conn, None).await?;
    shared::tracing::info!("Migrations done!");


    
    shared::tracing::info!("Initializing default groups...");
    let mut node = RustpermsMasterProtoClient
        ::connect(format!("http://{}:{}", ENV.RUSTPERMS_MASTER_ADDR, ENV.RUSTPERMS_MASTER_PORT)).await
        .inspect_err(|e|tracing::error!("Can't establish connection with master!: {e}"))?;

    let mut ops =  perms::groups::init_default();
    ops.extend(perms::user::profile::grant_default().into_iter());
    let delta = rustperms::prelude::RustpermsDelta::from(ops);
    node.write_changes(WriteRequest{serialized_delta: delta.serialize_to_string()?}).await?;
    shared::tracing::info!("Default groups initialized!");

    Ok(())
}
