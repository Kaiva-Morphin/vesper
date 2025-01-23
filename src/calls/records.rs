use std::{collections::HashMap, ops::{Deref, DerefMut}};
use axum_extra::extract::CookieJar;
use rand::random;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{now, ExtraCookie};

const ANONYMOUS_EXPIRATION: i64 = 60 * 60 * 32;
const ROOM_EXPIRATION: i64 = 60 * 60 * 5;

pub type UserId = u128;
pub type PublicUserId = UserId;
pub type RoomId = u128;

pub trait RoomIdExt {
    fn is_public(&self) -> bool;
    fn private() -> Self;
    fn public() -> Self;
    fn get_public(public: bool) -> Self;
}

impl RoomIdExt for RoomId {
    fn is_public(&self) -> bool {
        (self & 1) != 0
    }
    fn private() -> Self {
        random::<RoomId>() & !1
    }
    fn public() -> Self {
        random::<RoomId>() | 1
    }
    fn get_public(public: bool) -> Self {
        if public {
            Self::public()
        } else {
            Self::private()
        }
    }
}

pub const COOKIE_PUBLIC_ID: &'static str = "public_id";
pub const COOKIE_PRIVATE_ID: &'static str = "private_id";
pub const COOKIE_ROOM_ID: &'static str = "room_id";
pub const COOKIE_PASSWORD_HASH: &'static str = "passwd_hash";


#[derive(Default, Clone)]
pub struct AnonymousUser {
    pub id: UserId,
    pub public_id: PublicUserId,
    pub avatar: Option<String>,
    pub nickname: Option<String>,
    pub last_seen: i64,
}

impl AnonymousUser {
    pub fn new() -> Self {
        Self {
            id: random::<UserId>(),
            public_id : random::<PublicUserId>(),
            avatar: None,
            nickname: None,
            last_seen: 0,
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct Room {
    pub id: RoomId,
    pub owner: UserId,
    pub public_owner: String,
    pub max_users: u8,
    pub users: Vec<PublicUserId>,
    pub name: String,
    pub description: Option<String>,
    pub password: Option<String>,
    pub empty_since: i64,
}

#[derive(Default, Clone, Serialize)]
pub struct RoomInfo {
    pub id: RoomId,
    pub owner: String,
    pub max_users: u8,
    pub users: u8,
    pub name: String,
    pub description: Option<String>,
}

impl Room {
    pub fn as_info(&self) -> RoomInfo {
        RoomInfo {
            id: self.id,
            owner: self.public_owner.clone(),
            max_users: self.max_users,
            users: self.users.len() as u8,
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Default, Clone)]
pub struct RoomPool {
    pub private_rooms: HashMap<RoomId, Room>,
    pub public_rooms: HashMap<RoomId, Room>,
}

impl RoomPool {
    pub fn has_room(&self, room_id: RoomId) -> bool {
        if room_id.is_public() {
            self.public_rooms.contains_key(&room_id)
        } else {
            self.private_rooms.contains_key(&room_id)
        }
    }

    pub fn get_room(&self, room_id: RoomId) -> Option<Room> {
        if room_id.is_public() {
            self.public_rooms.get(&room_id).cloned()
        } else {
            self.private_rooms.get(&room_id).cloned()
        }
    }

    pub fn get_room_mut(&mut self, room_id: RoomId) -> Option<&mut Room> {
        if room_id.is_public() {
            self.public_rooms.get_mut(&room_id)
        } else {
            self.private_rooms.get_mut(&room_id)
        }
    }
    pub fn remove_room(&mut self, room_id: RoomId) -> Option<Room> {
        if room_id.is_public() {
            self.public_rooms.remove(&room_id)
        } else {
            self.private_rooms.remove(&room_id)
        }
    }

    pub fn add_room(&mut self, room: Room) {
        if room.id.is_public() {
            self.public_rooms.insert(room.id, room);
        } else {
            self.private_rooms.insert(room.id, room);
        }
    }
}

#[derive(Default, Clone)]
pub struct UserPool {
    pub users: HashMap<UserId, AnonymousUser>,
}

impl Deref for UserPool {
    type Target = HashMap<UserId, AnonymousUser>;
    fn deref(&self) -> &Self::Target {
        &self.users
    }
}

impl DerefMut for UserPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.users
    }
}

#[derive(Default)]
pub struct AppState {
    pub rooms: RwLock<RoomPool>,
    pub users: RwLock<UserPool>,
    pub sockets: RwLock<HashMap<UserId, UserId>>,
    pub room_users: RwLock<HashMap<RoomId, Vec<UserId>>>,
}

impl AppState {
    pub async fn is_user_valid(&self, user_id: UserId) -> bool {
        let mut users = self.users.write().await;
        let Some(user) = users.get_mut(&user_id) else { return false };
        if now() - user.last_seen > ANONYMOUS_EXPIRATION {
            users.remove(&user_id);
            return false;
        }
        user.last_seen = now();
        true
    }

    pub async fn get_visible_rooms(&self) -> Vec<RoomInfo> {
        let mut rooms = vec![];
        let mut to_delete = vec![];
        for room in self.rooms.read().await.public_rooms.values() {
            if room.empty_since != 0 && now() - room.empty_since > ROOM_EXPIRATION {
                to_delete.push(room.id);
                continue;
            }
            rooms.push(room.as_info());
        }
        if to_delete.len() > 0 {
            let mut rooms = self.rooms.write().await;
            for room_id in to_delete {
                rooms.public_rooms.remove(&room_id);
            }
        }
        rooms
    }

    pub async fn validate_or_create_user(&self, user_id: UserId) -> Result<(), AnonymousUser> {
        if self.is_user_valid(user_id).await {
            return Ok(());
        }
        let mut users: tokio::sync::RwLockWriteGuard<'_, UserPool> = self.users.write().await;
        let new_user = AnonymousUser::new();
        users.insert(user_id, new_user.clone());
        Err(new_user)
    }

    pub async fn jar_validate_or_create_user(&self, jar: CookieJar) -> CookieJar {
        let user_id = jar.get_user_id();
        if let Some(user_id) = user_id {
            if self.is_user_valid(user_id).await {return jar}
        }
        let new_user = AnonymousUser::new();
        let mut users = self.users.write().await;
        users.insert(new_user.id, new_user.clone());
        jar.put_user(new_user)
    }

    pub async fn is_room_valid(&self, room_id: RoomId) -> bool {
        self.get_room(room_id).await.is_some()
    }

    pub async fn get_room(&self, room_id: RoomId) -> Option<Room> {
        let Some(room) = self.rooms.read().await.get_room(room_id) else { return None };
        if (room.empty_since != 0) && (now() - room.empty_since > ROOM_EXPIRATION) {
            self.rooms.write().await.remove_room(room.id);
            return None;
        }
        Some(room)
    }

    pub async fn is_user_in_room(&self, user_id: UserId, room_id: RoomId) -> bool {
        let room_users = self.room_users.read().await;
        let Some(users) = room_users. get(&room_id) else { return false };
        users.contains(&user_id)
    }

    pub async fn add_user_to_room(&self, user_id: UserId, room_id: RoomId) {
        let mut room_users = self.room_users.write().await;
        let users = room_users.entry(room_id).or_insert(vec![]);
        users.push(user_id);
    }
}