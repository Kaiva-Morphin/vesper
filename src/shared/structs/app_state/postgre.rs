use deadpool_diesel::postgres::{Manager, Pool as PGPool};
use diesel::PgConnection;
use reqwest::StatusCode;

use crate::shared::{env::*, errors::adapt_error};




#[derive(Clone)]
pub struct Postgre {
    pool: PGPool
}

impl Postgre {
    pub fn default() -> Self {
        let manager = Manager::new(
            DATABASE_URL.to_string(),
            deadpool_diesel::Runtime::Tokio1,
        );
        Self {
            pool: PGPool::builder(manager).build().expect("Can't create pool for postgre!")
        }
    }

    pub async fn interact<V>(&self, action: impl Fn(&mut PgConnection) -> Result<V, StatusCode> + Send + 'static) -> Result<V, StatusCode>
    where
        V: Send + 'static
    {
        let conn = self.pool.get().await.map_err(adapt_error)?;
        conn.interact(action).await.map_err(adapt_error)?
    }
}
