
use std::{collections::HashMap, fs, str::FromStr, sync::Arc};
use axum::{
    extract::{Query, State}, response::{IntoResponse, Redirect}, routing::{get, post}, Json, Router
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;
use rand::random;
use reqwest::StatusCode;

use crate::calls::records::{AppState, RoomId};

use super::{offers::CreateRoom, records::{AnonymousUser, PublicUserId, Room, UserId, COOKIE_PASSWORD_HASH, COOKIE_PRIVATE_ID, COOKIE_PUBLIC_ID, COOKIE_ROOM_ID}};

pub async fn create_anonymous_user(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> CookieJar {
    state.jar_validate_or_create_user(jar).await
}

pub async fn get_rooms(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(state.get_visible_rooms().await)
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    mut jar: CookieJar,
    payload: Json<CreateRoom>
) -> Result<(CookieJar, Redirect), StatusCode> {
    jar = state.jar_validate_or_create_user(jar).await;
    let user_id = jar.get_user_id().ok_or_else(|| StatusCode::UNAUTHORIZED)?;
    let users = state.users.read().await;
    let user = users.get(&user_id).ok_or_else(|| StatusCode::UNAUTHORIZED)?;
    let public_owner = user.nickname.clone().unwrap_or(user.public_id.to_string());
    let room = payload.as_room(user_id, public_owner).ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let jar = jar.add(Cookie::new(COOKIE_ROOM_ID, room.id.to_string()));
    let mut rooms = state.rooms.write().await;
    rooms.public_rooms.insert(room.id, room);
    Ok((jar, Redirect::to("/call")))
}

pub trait ExtraCookie {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T>;
    fn put_user(self, user: AnonymousUser) -> Self;
    fn get_passwd_hash(&self) -> Option<String>;
    fn get_user_id(&self) -> Option<UserId>;
    fn get_public_user_id(&self) -> Option<PublicUserId>;
    fn get_room_id(&self) -> Option<RoomId>;
}
impl ExtraCookie for CookieJar {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(name).and_then(|cookie| cookie.value().parse::<T>().ok())
    }
    fn put_user(self, user: AnonymousUser) -> Self {
        self.add(Cookie::new(COOKIE_PUBLIC_ID, user.public_id.to_string()))
            .add(Cookie::new(COOKIE_PRIVATE_ID, user.id.to_string()))
    }
    fn get_passwd_hash(&self) -> Option<String> {
        self.get(COOKIE_PASSWORD_HASH).map(|v| v.value().to_string())
    }
    fn get_user_id(&self) -> Option<UserId> {
        self.get_typed(COOKIE_PRIVATE_ID)
    }
    fn get_public_user_id(&self) -> Option<PublicUserId> {
        self.get_typed(COOKIE_PUBLIC_ID)
    }
    fn get_room_id(&self) -> Option<RoomId> {
        self.get_typed(COOKIE_ROOM_ID)
    }
}

pub async fn join_room(
    State(state): State<Arc<AppState>>,
    mut jar: CookieJar
) -> Result<(CookieJar, Redirect), StatusCode> {
    jar = state.jar_validate_or_create_user(jar).await;
    let room_id = jar.get_room_id().ok_or_else(|| StatusCode::NOT_FOUND)?;
    let room = state.get_room(room_id).await.ok_or_else(|| StatusCode::NOT_FOUND)?;
    if room.password == jar.get_passwd_hash() {
        //let rooms = state.rooms.read().await;
        //let room = rooms.public_rooms.get(&room_id).ok_or_else(|| StatusCode::NOT_FOUND)?;
        successful_join();
        return Err(StatusCode::UNAUTHORIZED);
    } else {
        return Ok((jar, Redirect::to("/call/auth")))
    }
    
    // check is user already in room
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn successful_join(){}