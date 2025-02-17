use chrono::Utc;
use r2d2::PooledConnection;
use redis::{Client, Commands, FromRedisValue, RedisError, RedisResult};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{auth::oauth::shared::CRFS_LIFETIME, shared::{env::REDIS_URL, errors::{adapt_error, AsStatusCode}, settings::{MAX_LIVE_SESSIONS, REFRESH_TOKEN_LIFETIME}, structs::tokens::tokens::RefreshTokenRecord}};

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

const REFRESH_TOKEN_PREFIX : &'static str = "RTID";
const USER_TOKEN_PAIR_PREFIX : &'static str = "UTPP";
const CRFS_TOKEN_PREFIX : &'static str = "CRFS";
//const TEMPORARY_USERDATA_TOKEN_PREFIX : &'static str = "TMPR";

fn rtid_to_key(rtid: Uuid) -> String{
    format!("{}::{}", REFRESH_TOKEN_PREFIX, rtid)
}

fn user_to_key(user: Uuid) -> String{
    format!("{}::{}", USER_TOKEN_PAIR_PREFIX, user)
}

fn crfs_to_key(crfs: &String) -> String{
    format!("{}::{}", CRFS_TOKEN_PREFIX, crfs)
}

/*fn tmpr_to_key(tmpr: Uuid) -> String{
    format!("{}::{}", TEMPORARY_USERDATA_TOKEN_PREFIX, tmpr)
}*/


impl RedisTokens {
    pub fn default() -> Self {
        let redis_client = redis::Client::open(format!("{}/{}", REDIS_URL.to_string(), REDIS_TOKEN_DB)).expect("Can't connect to redis!");
        RedisTokens{
            pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }
    }
    
    pub fn set_refresh(&self, record: RefreshTokenRecord) -> Result<(), StatusCode>
    {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        let now = Utc::now().timestamp();
        let user_key = user_to_key(record.user);
        let valid_values: Vec<String> = conn.zrangebyscore(user_key.clone(), now, "+inf").map_err(adapt_error)?;
        if valid_values.len() >= MAX_LIVE_SESSIONS { // ERASE ALL SESSIONS
            for rtid_key in valid_values {
                let _: Result<(), RedisError> = conn.del(rtid_key);
            }
            let _: () = conn.zrembyscore(user_key.clone(), "-inf", "+inf").map_err(adapt_error)?;
        } else { // ERASE OUTDATED SESSIONS
            let _: () = conn.zrembyscore(user_key.clone(), "-inf", now).map_err(adapt_error)?;
        }
        let _: () = conn.zadd(user_key.clone(), rtid_to_key(record.rtid), now + REFRESH_TOKEN_LIFETIME as i64).map_err(adapt_error)?;
        let _: () = conn.set_ex(rtid_to_key(record.rtid), serde_json::to_string(&record).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?, REFRESH_TOKEN_LIFETIME).map_err(adapt_error)?;
        Ok(())
    }

    pub fn get_refresh(&self, rtid: Uuid) -> Result<Option<RefreshTokenRecord>, StatusCode>
    {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        self.get_refresh_conn(rtid, &mut conn)
    }

    fn get_refresh_conn(&self, rtid: Uuid, conn : &mut PooledConnection<Client>) -> Result<Option<RefreshTokenRecord>, StatusCode>
    {
        let s : Option<String> = conn.get(rtid_to_key(rtid)).map_err(adapt_error)?;
        let Some(s) = s else {return Ok(None)};
        let v = serde_json::from_str(s.as_str()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(v)
    }

    pub fn rm_refresh(&self, rtid: Uuid) -> Result<(), StatusCode> {
        let rtid_key = rtid_to_key(rtid);
        let mut conn = self.pool.get().map_err(adapt_error)?;
        if let Ok(Some(record)) = self.get_refresh_conn(rtid, &mut conn) {
            let _: Result<(), RedisError> = conn.zrem(user_to_key(record.user), rtid_key.clone());
        }
        let _: Result<(), RedisError> = conn.del(rtid_key);
        Ok(())
    }

    pub fn pop_refresh(&self, rtid: Uuid) -> Result<Option<RefreshTokenRecord>, StatusCode>
    {
        let rtid_key = rtid_to_key(rtid);
        let mut conn = self.pool.get().map_err(adapt_error)?;
        if let Ok(record) = self.get_refresh_conn(rtid, &mut conn) {
            let Some(record) = record else {return Ok(None)};
            let _: Result<(), RedisError> = conn.zrem(user_to_key(record.user), rtid_key.clone());
            let _: Result<(), RedisError> = conn.del(rtid_key);
            return Ok(Some(record))
        }
        let _: Result<(), RedisError> = conn.del(rtid_key);
        Err(StatusCode::UNAUTHORIZED)
    }

    pub fn set_crfs(
        &self,
        crfs: &String,
        provider: String
    ) -> Result<(), StatusCode> {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        let _ : () = conn.set_ex(crfs_to_key(crfs), provider, CRFS_LIFETIME).map_err(adapt_error)?;
        Ok(())
    }

    pub fn get_crfs(
        &self,
        crfs: &String
    ) -> Result<Option<String>, StatusCode> {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        let v : Option<String> = conn.get(crfs_to_key(crfs)).map_err(adapt_error)?;
        Ok(v)
    }

    pub fn pop_crfs(
        &self,
        crfs: &String
    ) -> Result<Option<String>, StatusCode> {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        let crfs_key = crfs_to_key(crfs);
        let v : Option<String> = conn.get(crfs_key.clone()).map_err(adapt_error)?;
        let _ : () = conn.del(crfs_key).map_err(adapt_error)?;
        Ok(v)
    }

    /*pub fn set_tmpr(
        &self,
        tmpr: Uuid
    ) -> Result<(), StatusCode> {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        let _ : () = conn.set_ex(tmpr_to_key(tmpr), tmpr.to_string(), CRFS_LIFETIME).map_err(adapt_error)?;
        Ok(())
    }

    pub fn pop_tmpr(
        &self,
        tmpr: Uuid
    ) -> Result<Option<String>, StatusCode> {
        let mut conn = self.pool.get().map_err(adapt_error)?;
        let tmpr_key = tmpr_to_key(tmpr);
        let v : Option<String> = conn.get(tmpr_key.clone()).map_err(adapt_error)?;
        let _ : () = conn.del(tmpr_key).map_err(adapt_error)?;
        Ok(v)
    }*/
}