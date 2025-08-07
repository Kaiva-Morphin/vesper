use shared::env_config;

pub mod publisher;
pub mod email;


env_config!(
    ".env" => ENV = Env {
        NATS_URL : String,
        NATS_PORT : String,
    }
);