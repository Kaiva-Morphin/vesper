use axum::{
    body::Bytes, extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade}, http::HeaderMap, response::IntoResponse, routing::any, Router
};
use layers::{auth::AuthAccessLayer, rustperms::PermissionMiddlewareBuilder};
use redis_utils::redis::RedisConn;
use rustperms_nodes::{connect_replica, proto::rustperms_replica_proto_client::RustpermsReplicaProtoClient};
use serde::ser;
use shared::{env_config, router, utils::{header::get_user_agent, logger::init_logger}};
use tokio::time::sleep;
use tonic::transport::Channel;
use tower::ServiceBuilder;
use tracing::info;

use std::{ops::ControlFlow, time::Duration};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    catch_panic::CatchPanicLayer, cors::{Any, CorsLayer}, trace::{DefaultMakeSpan, TraceLayer}
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures_util::{sink::SinkExt, stream::StreamExt};

mod rooms;
mod state;
mod nats;
mod types;
use state::*;
use crate::rooms::*;
use crate::nats::run_consumer;

env_config!( ".env" => ENV = Env {
    SERVICE_CALLS_PORT: u16,
    NATS_URL: String,
    NATS_PORT: u16,
    CALLS_NATS_EVENT: String,
});


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut service = service::Service::begin();

    let nats_url = format!("nats://{}:{}", ENV.NATS_URL, ENV.NATS_PORT);



    let replica = connect_replica().await.unwrap();
    let state = AppState::new().await;
    let p = PermissionMiddlewareBuilder::new(replica);
    let default_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(layers::layer_with_unique_span!("request ")))
        .layer(axum::middleware::from_fn(layers::logging::logging_middleware))
        .layer(CatchPanicLayer::new());
    // TODO!: DEV ONLY
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:1420".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));


    // TODO! : DELETE IS DEV ONLY
    service.route(router!(
            p,
                "/api/calls": (AuthAccessLayer::allow_guests()) => {
                    delete "/rooms" -> delete_all_rooms ("calls.join")

                    any "/" -> connect ("calls.connect")

                    get "/rooms" -> get_rooms ("calls.view")
                    get "/all_rooms" -> get_all_rooms ("calls.view.hidden")
                }
            ).layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
            .layer(default_layer)
            .layer(cors)
            .with_state(state.clone())
    );

    tokio::spawn(async move {
        tracing::info!("Starting nats event listener...");
        loop {  
            let result = run_consumer(&state).await;
            if let Err(e) = result {
                tracing::error!("NATS consumer failed: {e}");
            }
            info!("NATS consumer stopped, restarting");
            sleep(Duration::from_secs(5)).await;
        }
    });

    service.run(ENV.SERVICE_CALLS_PORT).await?;
    Ok(())
}


