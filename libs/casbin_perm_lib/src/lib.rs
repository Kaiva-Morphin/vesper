use sqlx_adapter::SqlxAdapter;

pub mod middleware;

const INIT_SCRIPT : &'static str = "CREATE TABLE IF NOT EXISTS casbin_rule (
    id SERIAL PRIMARY KEY,
    ptype VARCHAR NOT NULL,
    v0 VARCHAR NOT NULL,
    v1 VARCHAR NOT NULL,
    v2 VARCHAR NOT NULL,
    v3 VARCHAR NOT NULL,
    v4 VARCHAR NOT NULL,
    v5 VARCHAR NOT NULL,
    CONSTRAINT unique_key_sqlx_adapter UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
);";

#[derive(Clone)]
pub struct PermissionDB(SqlxAdapter);

impl PermissionDB {
    pub async fn init(url: &str, pool_size: u32) -> anyhow::Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(url).await?;
        sqlx::query(INIT_SCRIPT).execute(&pool).await?;
        Ok(PermissionDB(SqlxAdapter::new_with_pool(pool).await?))
    }
}