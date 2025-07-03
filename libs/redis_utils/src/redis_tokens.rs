use bb8::PooledConnection;
use bb8_redis::{redis::{AsyncCommands, RedisError}, RedisConnectionManager};
use chrono::Utc;
// use redis::{Client, FromRedisValue, RedisError, RedisResult};
use reqwest::StatusCode;
use tracing::info;
use uuid::Uuid;
pub use redis::Commands;

use shared::tokens::jwt::RefreshTokenRecord;
use anyhow::Result;

use crate::{redis::{RedisConn, RedisTokens}, CFG, ENV};



const REFRESH_TOKEN_PREFIX : &'static str = "RTID";
const USER_TOKEN_PAIR_PREFIX : &'static str = "UTPP";
// const CRFS_TOKEN_PREFIX : &'static str = "CRFS";
//const TEMPORARY_USERDATA_TOKEN_PREFIX : &'static str = "TMPR";

fn rtid_to_key(rtid: Uuid) -> String{
    format!("{}::{}", REFRESH_TOKEN_PREFIX, rtid)
}   

fn user_to_key(user: Uuid) -> String{
    format!("{}::{}", USER_TOKEN_PAIR_PREFIX, user)
}



impl RedisTokens {
    pub async fn default() -> Self {
        RedisConn::tokens().await.into()
    }
    
    pub async fn set_refresh(&self, record: RefreshTokenRecord) -> Result<()>
    {
        let mut conn = self.pool.get().await?;
        let now = Utc::now().timestamp();
        let user_key = user_to_key(record.user);
        let valid_values: Vec<String> = conn.zrangebyscore(user_key.clone(), now, "+inf").await?;
        if valid_values.len() >= CFG.MAX_LIVE_SESSIONS { // ERASE ALL SESSIONS
            for rtid_key in valid_values {
                let _: Result<(), RedisError> = conn.del(rtid_key).await;
            }
            let _: () = conn.zrembyscore(user_key.clone(), "-inf", "+inf").await?;
        } else { // ERASE OUTDATED SESSIONS
            let _: () = conn.zrembyscore(user_key.clone(), "-inf", now).await?;
        }
        let _: () = conn.zadd(user_key.clone(), rtid_to_key(record.rtid), now + CFG.REDIS_REFRESH_TOKEN_LIFETIME as i64).await?;
        let _: () = conn.set_ex(rtid_to_key(record.rtid), serde_json::to_string(&record)?, CFG.REDIS_REFRESH_TOKEN_LIFETIME).await?;
        Ok(())
    }

    pub async fn get_refresh(&self, rtid: String) -> Result<Option<RefreshTokenRecord>>
    {
        let mut conn = self.pool.get().await?;
        self.get_refresh_conn(rtid, &mut conn).await
    }

    async fn get_refresh_conn(&self, rtid: String, conn : &mut PooledConnection<'_, RedisConnectionManager>) -> Result<Option<RefreshTokenRecord>>
    {
        let s : Option<String> = conn.get(rtid).await?;
        let Some(s) = s else {return Ok(None)};
        let v = serde_json::from_str(s.as_str())?;
        Ok(v)
    }

    pub async fn rm_refresh(&self, rtid: Uuid) -> Result<()> {
        let rtid_key = rtid_to_key(rtid);
        let mut conn = self.pool.get().await?;
        if let Ok(Some(record)) = self.get_refresh_conn(rtid_key.clone(), &mut conn).await {
            let _: Result<(), RedisError> = conn.zrem(user_to_key(record.user), rtid_key.clone()).await;
        }
        let _: Result<(), RedisError> = conn.del(rtid_key).await;
        Ok(())
    }

    pub async fn rm_all_refresh(&self, user: Uuid) -> Result<()> {
        let mut conn = self.pool.get().await?;
        info!("Removing all refresh tokens!");
        let user_key = user_to_key(user);
        let keys: Vec<String> = conn.zrangebyscore(user_key.clone(), "-inf", "+inf").await?;
        for rtid_key in keys {
            let _: Result<(), RedisError> = conn.del(rtid_key).await;
        }
        Ok(())
    }

    pub async fn pop_refresh(&self, rtid: Uuid) -> Result<Option<RefreshTokenRecord>>
    {
        info!("Popping refresh for {rtid}");
        let rtid_key = rtid_to_key(rtid);
        let mut conn = self.pool.get().await?;
        if let Ok(record) = self.get_refresh_conn(rtid_key.clone(), &mut conn).await {
            let Some(record) = record else {return Ok(None)};
            info!("Record found!");
            let _: Result<(), RedisError> = conn.zrem(user_to_key(record.user), rtid_key.clone()).await;
            let _: Result<(), RedisError> = conn.del(rtid_key).await;
            return Ok(Some(record))
        }
        // let _: Result<(), RedisError> = conn.del(rtid_key);
        info!("No record!");
        Ok(None)
    }

    // pub async fn set_crfs(
    //     &self,
    //     crfs: &String,
    //     provider: String
    // ) -> Result<(), StatusCode> {
    //     let mut conn = self.pool.get()?;
    //     let _ : () = conn.set_ex(crfs_to_key(crfs), provider, CRFS_LIFETIME)?;
    //     Ok(())
    // }

    // pub async fn get_crfs(
    //     &self,
    //     crfs: &String
    // ) -> Result<Option<String>, StatusCode> {
    //     let mut conn = self.pool.get()?;
    //     let v : Option<String> = conn.get(crfs_to_key(crfs))?;
    //     Ok(v)
    // }

    // pub async fn pop_crfs(
    //     &self,
    //     crfs: &String
    // ) -> Result<Option<String>, StatusCode> {
    //     let mut conn = self.pool.get()?;
    //     let crfs_key = crfs_to_key(crfs);
    //     let v : Option<String> = conn.get(crfs_key.clone())?;
    //     let _ : () = conn.del(crfs_key)?;
    //     Ok(v)
    // }

    /*pub async fn set_tmpr(
        &self,
        tmpr: Uuid
    ) -> Result<(), StatusCode> {
        let mut conn = self.pool.get()?;
        let _ : () = conn.set_ex(tmpr_to_key(tmpr), tmpr.to_string(), CRFS_LIFETIME)?;
        Ok(())
    }

    pub async fn pop_tmpr(
        &self,
        tmpr: Uuid
    ) -> Result<Option<String>, StatusCode> {
        let mut conn = self.pool.get()?;
        let tmpr_key = tmpr_to_key(tmpr);
        let v : Option<String> = conn.get(tmpr_key.clone())?;
        let _ : () = conn.del(tmpr_key)?;
        Ok(v)
    }*/
}