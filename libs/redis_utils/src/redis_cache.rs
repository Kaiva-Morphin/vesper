use crate::redis::RedisConn;
use anyhow::Result;

use serde::{Serialize, de::DeserializeOwned};
use bb8_redis::redis::{AsyncCommands};
use tracing::info;

pub trait RedisCache {
    async fn set<T: Serialize>(&self, key: impl AsRef<str>, value: T, ttl_seconds: u64) -> Result<()>;
    async fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<Option<T>>;
    async fn del(&self, key: impl AsRef<str>) -> Result<()>;
    async fn hset<T: Serialize>(&self, parent: impl AsRef<str>, field: impl AsRef<str>, value: T) -> Result<()>;
    async fn hget<T: DeserializeOwned>(&self, parent: impl AsRef<str>, field: impl AsRef<str>) -> Result<Option<T>>;
    async fn hdel(&self, parent: impl AsRef<str>, field: impl AsRef<str>) -> Result<()>;
    async fn hgetall<T: DeserializeOwned>(&self, parent: impl AsRef<str>) -> Result<std::collections::HashMap<String, T>>;
}

impl RedisCache for RedisConn {
    async fn set<T: Serialize>(&self, key: impl AsRef<str>, value: T, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let json = serde_json::to_string(&value)?;
        let _ : () = conn.set_ex(key.as_ref(), json, ttl_seconds).await?;
        Ok(())
    }

    async fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<Option<T>> {
        let mut conn = self.pool.get().await?;
        let json : Option<String> = conn.get(key.as_ref()).await?;
        if let Some(json) = json {
            let value = serde_json::from_str(&json)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn del(&self, key: impl AsRef<str>) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _ : () = conn.del(key.as_ref()).await?;
        Ok(())
    }

    async fn hset<T: Serialize>(
        &self,
        parent: impl AsRef<str>,
        field: impl AsRef<str>,
        value: T
    ) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let json = serde_json::to_string(&value)?;
        info!("PUTTED JSON: {}", json);
        let _: () = conn.hset(parent.as_ref(), field.as_ref(), json).await?;
        Ok(())
    }

    async fn hget<T: DeserializeOwned>(
        &self,
        parent: impl AsRef<str>,
        field: impl AsRef<str>
    ) -> Result<Option<T>> {
        let mut conn = self.pool.get().await?;
        let json: Option<String> = conn.hget(parent.as_ref(), field.as_ref()).await?;
        Ok(json.map(|j| serde_json::from_str(&j)).transpose()?)
    }

    async fn hgetall<T: DeserializeOwned>(
        &self,
        parent: impl AsRef<str>
    ) -> Result<std::collections::HashMap<String, T>> {
        let mut conn = self.pool.get().await?;
        let data: std::collections::HashMap<String, String> =
            conn.hgetall(parent.as_ref()).await?;
        let parsed = data
            .into_iter()
            .map(|(k, v)| Ok((k, serde_json::from_str(&v)?)))
            .collect::<Result<_>>()?;
        Ok(parsed)
    }

    async fn hdel(&self, parent: impl AsRef<str>, field: impl AsRef<str>) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _: () = conn.hdel(parent.as_ref(), field.as_ref()).await?;
        Ok(())
    }
}
