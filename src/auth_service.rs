use std::time::Duration;

use auth::endpoints::{login::login, register::register};
use auth::endpoints::username::check_username;
use axum::error_handling::HandleErrorLayer;
use axum::routing::{get, post};
use axum::{BoxError, Router};
use diesel;
use reqwest::StatusCode;

pub mod shared;
pub mod models;
pub mod schema;
pub mod auth;
use shared::env::SERVICE_AUTH_PORT;
use shared::structs::app_state::postgre::Postgre;
use shared::structs::app_state::redis_tokens::RedisTokens;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;


#[derive(Clone)]
pub struct AppState{
    postgre: Postgre, 
    tokens: RedisTokens,
}

#[tokio::main]
async fn main() {
    dotenvy::from_path(".env").ok();

    tracing_subscriber::fmt::init();

    let appstate = AppState{
        postgre: Postgre::default(),
        tokens: RedisTokens::default()
    };

    let app = Router::new()
        .route("/register", post(register))
        .route("/username_available", get(check_username))
        .route("/login", post(login))
        .route_layer(
            ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_too_many_requests))
            .layer(BufferLayer::new(1024))
            .layer(RateLimitLayer::new(1, Duration::from_secs(2)))
        )
        .with_state(appstate);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", SERVICE_AUTH_PORT.to_string())).await.unwrap();
    println!("{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}


async fn handle_too_many_requests(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("To many requests: {}", err)
    )
}