use std::sync::Arc;
use axum::{
    routing::{get, post}, Router
};

pub mod calls;
use calls::{endpoints::*, records::AppState};
use chrono::Utc;

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/get_rooms", get(get_rooms))
        .route("/create_room", post(create_room))
        .route("/create_anonymous_user", get(create_anonymous_user))
        .route("/room", get(join_room))
        .with_state(app_state)
        ;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


pub fn now() -> i64 {
    Utc::now().timestamp()
}