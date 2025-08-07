pub mod tokens;
pub mod utils;
pub use once_cell;
pub use dotenvy;
pub use uuid;
pub use tracing;

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
    ".env" => ENV = EnvConfig{
        TURNSTILE_SECRET : String,
    }
    ".cfg" => CFG = EnvCfg{
        MIN_NICKNAME_LENGTH : usize,
        MAX_NICKNAME_LENGTH : usize,
    }
);