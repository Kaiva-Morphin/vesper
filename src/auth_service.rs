use axum::{
    extract::Query, response::{IntoResponse, Redirect}, routing::{get, post}, Json, Router
};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse, AuthUrl, Client, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sea_orm::{Database, DatabaseConnection};
use serde::Deserialize;
use std::{env, sync::Arc};

mod auth;
mod shared;

use shared::env::*;

use auth::{oauth::{discord_oauth_client, google_oauth_client}, refresh::refresh_tokens};
use auth::login::login;
use auth::register::register;

// SECRETS
const REFRESH_TOKEN_SECRET : &[u8] = "refresh_secret".as_bytes();
const ACCESS_TOKEN_SECRET : &[u8] = "access_secret".as_bytes();
// VARIABLES

const CONTACT_ADMIN_MESSAGE : &'static str = "Notify the administrator if the error does not resolve itself after some time";


struct AppState {
    db: DatabaseConnection,
}

impl AppState {
    async fn default() -> Self {
        let db = Database::connect(DB_ADDRESS.as_str())
            .await
            .expect("Database connection failed");
        Self { db }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::from_path(".env").ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let auth_port = env::var("PORT").expect("PORT is not set in .env file");
    tracing_subscriber::fmt::init();


    let appstate = Arc::new(AppState::default());

    let app = Router::new()
        .route("/register", post(register))
        .route("/refresh_tokens", post(refresh_tokens))
        .route("/login", post(login))

        .route("/auth/google/login", get(google_login))
        .route("/auth/google/callback", get(google_callback))
        .route("/auth/discord/login", get(discord_login))
        .route("/auth/discord/callback", get(discord_callback))

        .with_state(appstate)
        ;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", SERVICE_AUTH_PORT.to_string())).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}









async fn google_login() -> impl IntoResponse {
    let client = google_oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();
    Redirect::temporary(auth_url.to_string().as_str())
}

#[derive(Deserialize)]
struct AuthCallback {
    code: String,
    state: String,
}

async fn google_callback(Query(params): Query<AuthCallback>) -> impl IntoResponse {
    let client = Client::new(ClientId::new(env::var("GOOGLE_CLIENT_ID").unwrap()))
        .set_client_secret(ClientSecret::new(GOOGLE_CLIENT_SECRET.to_string()))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(GOOGLE_REDIRECT_URI.to_string()).unwrap());
    let token = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(::oauth2::reqwest::async_http_client)
        .await
        .unwrap();
    format!("Access Token: {:?}\n Info: {:?}", token, fetch_google_user_info(token.access_token().secret()).await)
}

async fn discord_login() -> impl IntoResponse {
    let client = discord_oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();
    Redirect::temporary(auth_url.to_string().as_str())
}

async fn discord_callback(Query(params): Query<AuthCallback>) -> impl IntoResponse {
    let client = discord_oauth_client();
    let token = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(::oauth2::reqwest::async_http_client)
        .await
        .unwrap();
    format!("Access Token: {:?}\n Info: {:?}", token, fetch_discord_user_info(token.access_token().secret()).await)
}

async fn fetch_google_user_info(access_token: &str) -> Result<serde_json::Value, reqwest::Error> {
    let userinfo_url = "https://www.googleapis.com/oauth2/v3/userinfo";
    let client = reqwest::Client::new();

    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response)
}


async fn fetch_discord_user_info(access_token: &str) -> Result<serde_json::Value, reqwest::Error> {
    let userinfo_url = "https://discord.com/api/users/@me";
    let client = reqwest::Client::new();

    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response)
}



