

env_config! {
    "appstate.env" =>
    ENV_VARS = EnvConfig {
        CALLBACK_REDIRECT_URI: String,
        GOOGLE_CLIENT_ID: String,
        GOOGLE_REDIRECT_URI: String,
        GOOGLE_CLIENT_SECRET: String,
        DISCORD_CLIENT_ID: String,
        DISCORD_REDIRECT_URI: String,
        DISCORD_CLIENT_SECRET: String,
        DISCORD_AUTH_URI: String,
        ACCESS_TOKEN_SECRET: String,
        REFRESH_TOKEN_SECRET: String,
        TEMPORARY_USERDATA_TOKEN_SECRET: String,
        REDIS_URL: String,
        REDIS_PORT: u16,
        DATABASE_URL: String,
        DB_USERNAME: String,
        DB_PASSWORD: String,
        DB_ADDRESS: String,
        DB_PORT: u16,
        SERVICE_AUTH_PORT: u16,
    }
}