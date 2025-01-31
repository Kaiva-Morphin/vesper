use std::time::Duration;
use axum::{routing::get, Router, error_handling::HandleErrorLayer, BoxError, http::StatusCode};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/1", get(handler1))
        .route("/2", get(handler2))
        .route_layer(
            ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_too_many_requests))
            .layer(BufferLayer::new(1024))
            .layer(RateLimitLayer::new(1, Duration::from_secs(2)))
        )
        .route("/3", get(handler3));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn handler1() -> Result<String, StatusCode> {
    //tokio::time::sleep(Duration::from_secs(3)).await;
    Ok("handler1".to_owned())
}

async fn handler2() -> Result<String, StatusCode> {
    Ok("handler2".to_owned())
}

async fn handler3() -> Result<String, StatusCode> {
    Ok("handler3".to_owned())
}

async fn handle_too_many_requests(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("To many requests: {}", err)
    )
}