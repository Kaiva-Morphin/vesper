use std::time::Duration;

use auth::endpoints::register::get_criteria;
use auth::endpoints::token::refresh_tokens;
use auth::endpoints::{login::login, register::register};
use auth::endpoints::username::check_username;
use auth::oauth::any::auth_callback;
use auth::oauth::discord::{discord_callback, discord_login, discord_oauth_client};
use auth::oauth::google::{google_callback, google_login, google_oauth_client};
use axum::error_handling::HandleErrorLayer;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{BoxError, Router};
use diesel;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl};
use reqwest::StatusCode;

pub mod shared;
pub mod models;
pub mod schema;
pub mod auth;
use shared::env::*;
use shared::structs::app_state::postgre::Postgre;
use shared::structs::app_state::redis_tokens::RedisTokens;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;


#[derive(Clone)]
pub struct AppState{
    pub google_client: BasicClient,
    pub discord_client: BasicClient,
    pub postgre: Postgre, 
    pub tokens: RedisTokens,
}

#[tokio::main]
async fn main() {
    dotenvy::from_path(".env").ok();

    tracing_subscriber::fmt::init();
    let appstate = AppState{
        google_client: google_oauth_client().expect("Can't create google client!"),
        discord_client: discord_oauth_client().expect("Can't create discord client!"),
        postgre: Postgre::default(),
        tokens: RedisTokens::default()
    };

    let app = Router::new()
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/criteria", post(get_criteria))
        .route("/api/v1/auth/username_available", get(check_username))
        .route("/api/v1/auth/refresh", post(refresh_tokens))
        .route("/api/v1/auth/login", post(login))
        // .route("api/auth/callback") // todo!
        .route("/api/v1/auth/discord", get(discord_login))
        .route("/api/v1/auth/google", get(google_login))
        .route("/api/v1/auth/callback", get(auth_callback))
        .route_layer(
            ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_too_many_requests))
            .layer(BufferLayer::new(1024))
            .layer(RateLimitLayer::new(1, Duration::from_secs(2)))
        )
        .with_state(appstate);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", SERVICE_AUTH_PORT.to_string())).await.unwrap();
    println!("{}", listener.local_addr().expect("failed to return local address"));
    axum::serve(listener, app).await.unwrap();
}


async fn handle_too_many_requests(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("To many requests: {}", err)
    )
}









