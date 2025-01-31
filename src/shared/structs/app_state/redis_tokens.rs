use redis::{Client, Commands, FromRedisValue, RedisError, RedisResult};
use reqwest::StatusCode;

use crate::shared::{env::REDIS_URL, errors::{adapt_error, AsStatusCode}};

use super::vars::REDIS_TOKEN_DB;

#[derive(Clone)]
pub struct RedisTokens{
    pool: r2d2::Pool<Client>
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


impl RedisTokens {
    pub fn default() -> Self {
        let redis_client = redis::Client::open(format!("{}/{}", REDIS_URL.to_string(), REDIS_TOKEN_DB)).expect("Can't connect to redis!");
        RedisTokens{
            pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }
    }
    pub async fn set(&self, key: String, value: String) -> Result<(), StatusCode>
    {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        conn.set(key, value).map_err(adapt_error)
    }
    pub async fn set_ex(&self, key: String, value: String, lifetime_secs: u64) -> Result<(), StatusCode>
    {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        conn.set_ex(key, value, lifetime_secs).map_err(adapt_error)
    }
    pub async fn get<T : FromRedisValue>(&self, key: String) -> Result<RedisResult<T>, StatusCode>
    {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        Ok(conn.get(key))
    }
}