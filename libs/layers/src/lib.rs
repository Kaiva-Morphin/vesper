use shared::env_config;

pub mod auth;
pub mod logging;

pub use uuid;
pub use tracing;

env_config!(
    ".env" => ENV = EnvConfig{
        REDIS_TOKEN_DB : String = "4".to_string(),
        REDIS_PORT : u16,
        REDIS_URL : String,
        TURNSTILE_SECRET : String,
    }
    ".cfg" => CFG = EnvCfg{
        REDIS_REFRESH_TOKEN_LIFETIME : u64 = 30 * 24 * 60 * 60, // 30 days
        REDIS_MAX_LIVE_SESSIONS : usize = 5,
    }
);