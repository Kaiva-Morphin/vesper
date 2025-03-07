pub mod tokens;
pub mod utils;
pub mod layers;
pub mod env;
pub use once_cell;
pub use dotenvy;
use once_cell::sync::Lazy;

#[macro_export]
macro_rules! default_err {
    () => {
        Err(anyhow::Error::msg("Error"))
    };
    ($msg:expr) => {
        Err(anyhow::Error::msg($msg))
    };
}

env_config!(
    "shared.env" => ENV = EnvConfig{
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