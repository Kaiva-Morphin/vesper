pub mod users;
pub mod redis_cache;
pub mod redis;
use shared::env_config;

pub mod redis_tokens;
// pub mod redis_perms;

env_config!(
    ".env" => ENV = EnvConfig{
        // REDIS_TOKEN_DB : u8 = 4,
        // REDIS_PERMS_DB : u8 = 5,
        REDIS_PORT : u16,
        REDIS_URL : String,
    }
    ".cfg" => CFG = EnvCfg{
        REDIS_REFRESH_TOKEN_LIFETIME : u64 = 30 * 24 * 60 * 60, // 30 days
        MAX_LIVE_SESSIONS : usize = 5,
    }
);