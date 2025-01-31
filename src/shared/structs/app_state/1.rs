use deadpool_diesel::postgres::{Manager, Pool as PGPool};
use diesel::PgConnection;
use redis::{Client, Commands, ConnectionLike, FromRedisValue, RedisError, RedisResult};
use reqwest::StatusCode;

use crate::shared::{env::*, errors::{adapt_error, AsStatusCode}};


#[derive(Clone)]
pub struct AppState {
    db_pool: PGPool,
    redis_pool: r2d2::Pool<Client>,
}

impl AsStatusCode for r2d2::Error {
    fn as_interaction_error(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl AsStatusCode for RedisError {
    fn as_interaction_error(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl AppState {
    pub fn default() -> Self {
        let manager = Manager::new(
            DATABASE_URL.to_string(),
            deadpool_diesel::Runtime::Tokio1,
        );
        let redis_client = redis::Client::open(REDIS_URL.to_string()).expect("Can't connect to redis!");
        AppState{
            db_pool: PGPool::builder(manager).build().expect("Can't create pool for postgre!"),
            redis_pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }
    }
    pub fn redis_for_tokens() -> Self {
        let manager = Manager::new(
            DATABASE_URL.to_string(),
            deadpool_diesel::Runtime::Tokio1,
        );
        let redis_client = redis::Client::open(format!("{}/{}", REDIS_URL.to_string(), REDIS_TOKEN_DB.to_string())).expect("Can't connect to redis!");
        AppState{
            db_pool: PGPool::builder(manager).build().expect("Can't create pool for postgre!"),
            redis_pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }
    }
    
    pub async fn redis_set(&self, key: String, value: String) -> Result<(), StatusCode>
    {
        let mut conn = self.redis_pool.get().map_err(adapt_error)?;
        conn.set(key, value).map_err(adapt_error)
    }
    pub async fn redis_set_ex(&self, key: String, value: String, lifetime_secs: u64) -> Result<(), StatusCode>
    {
        let mut conn = self.redis_pool.get().map_err(adapt_error)?;
        conn.set_ex(key, value, lifetime_secs).map_err(adapt_error)
    }
    pub async fn redis_get<T : FromRedisValue>(&self, key: String) -> Result<RedisResult<T>, StatusCode>
    {
        let mut conn = self.redis_pool.get().map_err(adapt_error)?;
        Ok(conn.get(key))
    }
}

