use std::sync::Arc;

use axum::{
    extract::{Query, State}, response::{IntoResponse, Redirect}, routing::{get, post}, Json, Router
};
use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};


struct AppState {
    db: DatabaseConnection,
}

impl AppState {
    async fn default() -> Self {
        let db = Database::connect("postgresql://root@localhost:26257?sslmode=disable")
            .await
            .expect("Database connection failed");
        Self { db }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let appstate = Arc::new(AppState::default().await);

    let app = Router::new()
        .route("/", get(get_data))
        .route("/put", post(put_data))
        .with_state(appstate)
        ;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:33209")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub text: String,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[derive(Serialize, Deserialize)]
struct Data {
    key: String,
    value: String
}

async fn put_data(
    State(db): State<Arc<AppState>>,
    payload: Json<Data>
) -> impl IntoResponse {
    

    Json(())
}

async fn get_data(
    State(db): State<Arc<AppState>>
) -> impl IntoResponse {
    Json(())
}




