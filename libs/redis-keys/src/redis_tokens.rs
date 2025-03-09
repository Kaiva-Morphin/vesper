use chrono::Utc;
use r2d2::PooledConnection;
use redis::{Client, Commands, FromRedisValue, RedisError, RedisResult};
use reqwest::StatusCode;
use shared::{env_config, tokens::jwt::RefreshTokenRecord};
use uuid::Uuid;
use anyhow::Result;
//use crate::{auth::oauth::shared::CRFS_LIFETIME, shared::{env::REDIS_URL, errors::{adapt_error, AsStatusCode}, settings::{MAX_LIVE_SESSIONS, REFRESH_TOKEN_LIFETIME}, structs::tokens::tokens::RefreshTokenRecord}};




env_config! {
    "redis.env" =>
    CFG = EnvConfig {
        TOKEN_DB_PREFIX : String = "4".to_string(),
        CALLS_DB_PREFIX : String = "5".to_string(),
        MAX_LIVE_SESSIONS : usize = 5,
        REDIS_URL : String,
    }
}

env_config! {
    "shared.env" =>
    ENV = SharedEnv {
        REFRESH_TOKEN_LIFETIME : u64,
        CRFS_LIFETIME : u64,
    }
}


#[derive(Clone)]
pub struct RedisConn{
    pool: r2d2::Pool<Client>
}

// impl AsStatusCode for r2d2::Error {
//     fn as_interaction_error(&self) -> StatusCode {
//         StatusCode::INTERNAL_SERVER_ERROR
//     }
// }

// impl AsStatusCode for RedisError {
//     fn as_interaction_error(&self) -> StatusCode {
//         StatusCode::INTERNAL_SERVER_ERROR
//     }
// }

const REFRESH_TOKEN_PREFIX : &'static str = "RTID";
const USER_TOKEN_PAIR_PREFIX : &'static str = "UTPP";
const CRFS_TOKEN_PREFIX : &'static str = "CRFS";

fn rtid_to_key(rtid: Uuid) -> String{
    format!("{}::{}", REFRESH_TOKEN_PREFIX, rtid)
}

fn user_to_key(user: Uuid) -> String{
    format!("{}::{}", USER_TOKEN_PAIR_PREFIX, user)
}

fn crfs_to_key(crfs: &String) -> String{
    format!("{}::{}", CRFS_TOKEN_PREFIX, crfs)
}

impl RedisConn {
    pub fn default() -> Self {
        let redis_client = redis::Client::open(format!("{}/{}", CFG.REDIS_URL.to_string(), CFG.TOKEN_DB_PREFIX)).expect("Can't connect to redis!");
        RedisConn{
            pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }
    }
    
    pub fn set_refresh(&self, record: RefreshTokenRecord) -> Result<()>
    {
        let mut conn = self.pool.get()?;
        let now = Utc::now().timestamp();
        let user_key = user_to_key(record.rtid);
        let valid_values: Vec<String> = conn.zrangebyscore(user_key.clone(), now, "+inf")?;
        if valid_values.len() >= CFG.MAX_LIVE_SESSIONS { // ERASE ALL SESSIONS
            for rtid_key in valid_values {
                let _: Result<(), RedisError> = conn.del(rtid_key);
            }
            let _: () = conn.zrembyscore(user_key.clone(), "-inf", "+inf")?;
        } else { // ERASE OUTDATED SESSIONS
            let _: () = conn.zrembyscore(user_key.clone(), "-inf", now)?;
        }
        let _: () = conn.zadd(user_key.clone(), rtid_to_key(record.rtid), now + ENV.REFRESH_TOKEN_LIFETIME as i64)?;
        let _: () = conn.set_ex(rtid_to_key(record.rtid), record.rtid.to_string(), ENV.REFRESH_TOKEN_LIFETIME)?;
        Ok(())
    }

    pub fn get_refresh(&self, rtid: Uuid) -> Result<Option<RefreshTokenRecord>>
    {
        let mut conn = self.pool.get()?;
        self.get_refresh_conn(rtid, &mut conn)
    }

    fn get_refresh_conn(&self, rtid: Uuid, conn : &mut PooledConnection<Client>) -> Result<Option<RefreshTokenRecord>>
    {
        let s : Option<String> = conn.get(rtid_to_key(rtid))?;
        let Some(s) = s else {return Ok(None)};
        let v = serde_json::from_str(s.as_str())?;
        Ok(v)
    }

    pub fn rm_refresh(&self, rtid: Uuid) -> Result<()> {
        let rtid_key = rtid_to_key(rtid);
        let mut conn = self.pool.get()?;
        if let Ok(Some(record)) = self.get_refresh_conn(rtid, &mut conn) {
            let _: Result<(), RedisError> = conn.zrem(user_to_key(record.user), rtid_key.clone());
        }
        let _: Result<(), RedisError> = conn.del(rtid_key);
        Ok(())
    }

    pub fn pop_refresh(&self, rtid: Uuid) -> Result<Option<RefreshTokenRecord>>
    {
        let rtid_key = rtid_to_key(rtid);
        let mut conn = self.pool.get()?;
        if let Ok(record) = self.get_refresh_conn(rtid, &mut conn) {
            let Some(record) = record else {return Ok(None)};
            let _: Result<(), RedisError> = conn.zrem(user_to_key(record.user), rtid_key.clone());
            let _: Result<(), RedisError> = conn.del(rtid_key);
            return Ok(Some(record))
        }
        let _: Result<(), RedisError> = conn.del(rtid_key);
        Ok(None)
    }

    pub fn set_crfs(
        &self,
        crfs: &String,
        provider: String
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let _ : () = conn.set_ex(crfs_to_key(crfs), provider, ENV.CRFS_LIFETIME)?;
        Ok(())
    }

    pub fn get_crfs(
        &self,
        crfs: &String
    ) -> Result<Option<String>> {
        let mut conn = self.pool.get()?;
        let v : Option<String> = conn.get(crfs_to_key(crfs))?;
        Ok(v)
    }

    pub fn pop_crfs(
        &self,
        crfs: &String
    ) -> Result<Option<String>> {
        let mut conn = self.pool.get()?;
        let crfs_key = crfs_to_key(crfs);
        let v : Option<String> = conn.get(crfs_key.clone())?;
        let _ : () = conn.del(crfs_key)?;
        Ok(v)
    }

    /*pub fn set_tmpr(
        &self,
        tmpr: Uuid
    ) -> Result<(), StatusCode> {
        let mut conn = self.pool.get()?;
        let _ : () = conn.set_ex(tmpr_to_key(tmpr), tmpr.to_string(), CRFS_LIFETIME)?;
        Ok(())
    }

    pub fn pop_tmpr(
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