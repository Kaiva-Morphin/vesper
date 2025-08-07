use migration::MigratorTrait;
use sea_orm::{ConnectOptions, DatabaseConnection};
use shared::env_config;
use anyhow::Result;
env_config!(
    ".env" => ENV = Env {
        DATABASE_URL : String
    }
);
pub async fn open_database_connection() -> Result<DatabaseConnection> {
    let mut options = ConnectOptions::new(&ENV.DATABASE_URL);
    options.sqlx_logging(true); // TODO: ENABLE
    shared::tracing::info!("Connecting to database...");
    let conn = sea_orm::Database::connect(options).await?;
    shared::tracing::info!("Connected to database");
    Ok(conn)
}