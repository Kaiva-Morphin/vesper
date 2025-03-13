use std::ops::{Deref, DerefMut};

use redis::Client;




#[derive(Clone)]
pub struct RedisConn{
    pub pool: r2d2::Pool<Client>
}



macro_rules! redis_wrapper {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            conn: RedisConn
        }

        impl Deref for $name {
            type Target = RedisConn;
            fn deref(&self) -> &Self::Target {
                &self.conn
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.conn
            }
        }

        impl From<RedisConn> for $name {
            fn from(value: RedisConn) -> Self {
                Self { conn: value }
            }
        }
    };
}

redis_wrapper!(RedisTokens);
redis_wrapper!(RedisPerms);