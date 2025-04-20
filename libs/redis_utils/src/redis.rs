use std::ops::{Deref, DerefMut};

use bb8_redis::RedisConnectionManager;
use redis::Client;

use crate::ENV;




#[derive(Clone)]
pub struct RedisConn{
    pub pool: bb8::Pool<RedisConnectionManager>
}

impl RedisConn {
    pub async fn tokens() -> Self {
        Self::new(format!("redis://{}:{}/{}", ENV.REDIS_URL, ENV.REDIS_PORT, ENV.REDIS_TOKEN_DB)).await
    }
    pub async fn perms() -> Self {
        Self::new(format!("redis://{}:{}/{}", ENV.REDIS_URL, ENV.REDIS_PORT, ENV.REDIS_PERMS_DB)).await
    }
    pub async fn db(db: u8) -> Self {
        Self::new(format!("redis://{}:{}/{}", ENV.REDIS_URL, ENV.REDIS_PORT, db)).await
    }
    pub async fn new(conn_string: String) -> Self {
        let redis_client = RedisConnectionManager::new(conn_string).expect("Can't connect to redis!");
        RedisConn{
            pool: bb8::Pool::builder().build(redis_client).await.expect("Can't create pool for redis!")
        }
    }
}


#[macro_export]
macro_rules! redis_wrapper {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            conn: $crate::redis::RedisConn
        }

        impl std::ops::Deref for $name {
            type Target = $crate::redis::RedisConn;
            fn deref(&self) -> &Self::Target {
                &self.conn
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.conn
            }
        }

        impl From<$crate::redis::RedisConn> for $name {
            fn from(value: $crate::redis::RedisConn) -> Self {
                Self { conn: value }
            }
        }
    };
}

redis_wrapper!(RedisTokens);