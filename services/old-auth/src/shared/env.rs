use std::env;
use lazy_static::lazy_static;


macro_rules! static_env_var {
    ($name : expr) => {
        env::var($name).unwrap_or_else(|_| panic!("{} must be set in .env!", $name))
    };
}

macro_rules! parsed_static_env_var {
    ($name : expr) => {
        env::var($name).unwrap_or_else(|_| panic!("{} must be set in .env!", $name)).parse()
         .unwrap_or_else(|_| panic!("{} cant be parsed!", $name))
    };
}

lazy_static! {
    pub static ref CALLBACK_REDIRECT_URI : String = static_env_var!("CALLBACK_REDIRECT_URI");

    pub static ref GOOGLE_CLIENT_ID : String = static_env_var!("GOOGLE_CLIENT_ID");
    pub static ref GOOGLE_REDIRECT_URI : String = static_env_var!("GOOGLE_REDIRECT_URI");
    pub static ref GOOGLE_CLIENT_SECRET : String = static_env_var!("GOOGLE_CLIENT_SECRET");

    pub static ref DISCORD_CLIENT_ID : String = static_env_var!("DISCORD_CLIENT_ID");
    pub static ref DISCORD_REDIRECT_URI : String = static_env_var!("DISCORD_REDIRECT_URI");
    pub static ref DISCORD_CLIENT_SECRET : String = static_env_var!("DISCORD_CLIENT_SECRET");
    pub static ref DISCORD_AUTH_URI : String = static_env_var!("DISCORD_AUTH_URI");

    pub static ref ACCESS_TOKEN_SECRET : String = static_env_var!("ACCESS_TOKEN_SECRET");
    pub static ref REFRESH_TOKEN_SECRET : String = static_env_var!("REFRESH_TOKEN_SECRET");
    pub static ref TEMPORARY_USERDATA_TOKEN_SECRET : String = static_env_var!("TEMPORARY_USERDATA_TOKEN_SECRET");

    pub static ref REDIS_URL : String = static_env_var!("REDIS_URL");
    pub static ref REDIS_PORT : u32 = parsed_static_env_var!("REDIS_PORT");

    
    pub static ref DATABASE_URL : String = static_env_var!("DATABASE_URL");
    pub static ref DB_USERNAME : String = static_env_var!("DB_USERNAME");
    pub static ref DB_PASSWORD : String = static_env_var!("DB_PASSWORD");
    pub static ref DB_ADDRESS : String = static_env_var!("DB_ADDRESS");
    pub static ref DB_PORT : u32 = parsed_static_env_var!("DB_PORT");

    pub static ref SERVICE_AUTH_PORT : u32 = parsed_static_env_var!("SERVICE_AUTH_PORT");
}