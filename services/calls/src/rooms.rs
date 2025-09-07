use std::{net::SocketAddr, ops::ControlFlow, time::Duration};

use async_nats::Client;
use axum::{extract::{ws::{CloseFrame, Message, Utf8Bytes, WebSocket}, ConnectInfo, Query, State, WebSocketUpgrade}, http::HeaderMap, response::IntoResponse, Extension, Json};
use bytes::Bytes;
use chrono::Utc;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::AccessTokenPayload, utils::header::get_user_agent, uuid::Uuid};
use tokio::{sync::mpsc, time::interval};
use tracing::info;

use crate::{state::{RoomRecord, User, JS}, types::{CallEvent, ClientRequests, InnerSignal, Receiver}, AppState};

use anyhow::Result;



pub async fn get_rooms(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Ok(rooms) = state.get_rooms().await else {
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let r = rooms.into_values().map(|v| v.to_public()).collect::<Vec<_>>();
    (axum::http::StatusCode::OK, axum::Json(r)).into_response()
}

pub async fn get_all_rooms(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Ok((r1, r2)) = state.get_all_rooms().await else {
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let r = r1.into_values().chain(r2.into_values()).map(|v| v.to_public()).collect::<Vec<_>>();
    (axum::http::StatusCode::OK, axum::Json(r)).into_response()
}

fn to_user(data: Option<Extension<AccessTokenPayload>>, fallback: Option<String>) -> Option<User> {
    let Some(Extension(data)) = data else {
        return fallback.map(|s| User::Guest{guid: Uuid::new_v4().simple().to_string(), name: s});
    };
    Some(User::Logged{guid: data.user.simple().to_string()})
}


pub async fn delete_all_rooms(
    State(state): State<AppState>,
){
    state.delete_all_rooms().await;
}


#[derive(Deserialize)]
pub struct ConnectRequest {
    guest: Option<String>,
    token: Option<String>,
}

pub async fn connect(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    user: Option<Extension<AccessTokenPayload>>,
    Query(r): Query<ConnectRequest>,
) -> impl IntoResponse {
    let u = to_user(user, r.guest);
    let Some(u) = u else {return axum::http::StatusCode::BAD_REQUEST.into_response()};
    ws.on_upgrade(move |socket| handle_connect(socket, state, u))
}



pub trait WS {
    async fn send_close(self, msg: &str);
    async fn send_text(&mut self, msg: &str) -> Result<()>;
    async fn send_event(&mut self, event: CallEvent) -> Result<()>;
}

impl WS for WebSocket {
    async fn send_close(mut self, msg: &str) {
        self.send(Message::Close(Some(CloseFrame{reason: Utf8Bytes::from(msg), code: 1000}))).await.ok();
        self.close().await.ok();
    }

    async fn send_text(&mut self, msg: &str) -> Result<()> {
        self.send(Message::Text(Utf8Bytes::from(msg))).await?;
        Ok(())
    }
    async fn send_event(&mut self, event: CallEvent) -> Result<()> {
        self.send_text(&serde_json::to_string(&event)?).await?;
        Ok(())
    }
}

impl WS for SplitSink<WebSocket, Message> {
    async fn send_close(mut self, msg: &str) {
        self.send(Message::Close(Some(CloseFrame{reason: Utf8Bytes::from(msg), code: 1000}))).await.ok();
        self.close().await.ok();
    }
    async fn send_text(&mut self, msg: &str) -> Result<()> {
        self.send(Message::Text(Utf8Bytes::from(msg))).await?;
        Ok(())
    }
    async fn send_event(&mut self, event: CallEvent) -> Result<()> {
        self.send_text(&serde_json::to_string(&event)?).await?;
        Ok(())
    }
}


struct UserState {
    guid: String,
    user: User,
    room: Option<(String, bool)>,
    last_seen: i64
}


const TIMEOUT : i64 = 100;
const PING_INTERVAL : i64 = 5;
async fn handle_connect(mut socket: WebSocket, state: AppState, user: User) {
    if user.is_guest() {
        let s = serde_json::to_string(&user);
        let Ok(s) = s else {
            socket.send_close("Could not serialize user").await;
            return;
        };
        if socket.send_text(&s).await.is_err() {return;}
    }
    let guid = user.guid();
    let mut userstate : UserState = UserState{guid, user, room: None, last_seen: Utc::now().timestamp()}; 

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<InnerSignal>();
    state.signal_clients.write().await.insert(userstate.guid.clone(), tx);
    info!("Websocket context for {} created", userstate.user);

    let mut ping_interval = interval(Duration::from_secs(PING_INTERVAL as u64));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                if Utc::now().timestamp() - userstate.last_seen > TIMEOUT {
                    break;
                }
                if sender.send_event(CallEvent::Ping).await.is_err() {
                    break;
                }
            }
            Some(signal) = rx.recv() => {
                if !is_for_me(&userstate, &signal) {continue;}
                info!("Got signal: {:#?} for {}", signal.event, userstate.user);
                if sender.send_event(signal.event).await.is_err() {
                    break;
                }
            }
            msg = receiver.next() => {
                if msg.is_none() {break;}
                if process_user(&state, &mut userstate, msg).await.is_break() {
                    break;
                }
            }
        }
    }
    state.signal_clients.write().await.remove(&userstate.guid);
    if let Some((room, private)) = userstate.room {
        state.rm_user_from_room(&room, &private, &userstate.user).await.ok();
    }
    info!("Websocket context for {} destroyed", userstate.user);
}


fn is_for_me(userstate: &UserState, msg: &InnerSignal) -> bool {
    match &msg.rcv {
        Receiver::User(u) => &userstate.guid == u,
        Receiver::All => true,
        Receiver::Room(r) => userstate.room.as_ref().map(|(room, _)| room == r).unwrap_or(false)
    }
}


async fn process_user<E>(state: &AppState, userstate: &mut UserState, msg: Option<Result<Message, E>>) -> ControlFlow<()>{
    let Some(Ok(msg)) = msg else {return ControlFlow::Continue(())};
    let Message::Text(msg) = msg else {
        if let Message::Close(_) = msg {
            return ControlFlow::Break(());
        }
        return ControlFlow::Continue(())
    };
    let Ok(req) = serde_json::from_str::<ClientRequests>(&msg.to_string()) else {
        return ControlFlow::Continue(())
    };
    if let ClientRequests::Pong = req {} else {info!("Req: {:#?}!", req);};
    match req {
        ClientRequests::Pong => {
            userstate.last_seen = Utc::now().timestamp();
        }
        ClientRequests::Create { name, private, password } => {
            if let Some((room, private)) = &userstate.room {
                if state.rm_user_from_room(room, private, &userstate.user).await.is_err() {
                    return ControlFlow::Break(())
                };
                userstate.room = None;
            }
            let room = state.create_and_join_room(name, private.unwrap_or(false), password, userstate.user.clone()).await;
            let p = (room.guid.clone(), room.private);
            info!("Created and joined room {}", p.0);
            userstate.room = Some(p);
        }
        ClientRequests::Join { room_guid, password } => {
            if let Some((room, private)) = &userstate.room {
                if state.rm_user_from_room(room, private, &userstate.user).await.is_err() {
                    return ControlFlow::Break(())
                };
                userstate.room = None;
            }
            let room = state.try_join_room(&room_guid, &userstate.user, password).await;
            match room {
                Ok(r) => userstate.room = Some((r.guid, r.private)),
                Err(e) => {
                    state.jetstream.send_to_user(CallEvent::Error { msg: e.to_string() }, userstate.user.to_string()).await.ok();
                },
            }
        }
        ClientRequests::Leave => {
            if let Some((room, private)) = &userstate.room {
                if state.rm_user_from_room(room, private, &userstate.user).await.is_err() {
                    return ControlFlow::Break(())
                };
                userstate.room = None;
            }
        }
        ClientRequests::Message { msg } => {
            if let Some((room, _private)) = &userstate.room {
                state.jetstream.send_to_room(CallEvent::message(msg), room.to_string()).await.ok();
            }
        }
    }
    ControlFlow::Continue(())
}



