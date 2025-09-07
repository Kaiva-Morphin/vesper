use std::{collections::{HashMap, HashSet}, fmt::Display, sync::Arc};

use chrono::Utc;
use redis_utils::{redis::RedisConn, redis_cache::RedisCache};

use serde::{Deserialize, Serialize};
use shared::uuid::Uuid;

use anyhow::Result;
use tokio::sync::{mpsc, RwLock};
use tracing::error;

const PUBLIC_ROOMS_KEY : &str = "PUBLIC_ROOMS";
const PRIVATE_ROOMS_KEY : &str = "HIDDEN_ROOMS";

#[derive(Clone)]
pub struct AppState {
    pub redis: RedisConn,
    pub inbox: String,
    pub signal_clients: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<InnerSignal>>>>,
    pub jetstream: Arc<async_nats::jetstream::Context>,
}

pub(crate) trait JS {
    async fn send_to_all(&self, event: CallEvent) -> Result<()>;
    async fn send_to_room(&self, event: CallEvent, room: String) -> Result<()>;
    async fn send_to_user(&self, event: CallEvent, user: String) -> Result<()>;
}

impl JS for Arc<async_nats::jetstream::Context> {
    async fn send_to_all(
        &self, 
        event: CallEvent
    ) -> Result<()> {
        let i = InnerSignal{event, rcv: crate::types::Receiver::All};
        self.publish(ENV.CALLS_NATS_EVENT.clone(), serde_json::to_string(&i)?.into()).await?;
        Ok(())
    }
    async fn send_to_room(
        &self, 
        event: CallEvent,
        room: String
    ) -> Result<()> {
        let i = InnerSignal{event, rcv: crate::types::Receiver::Room(room)};
        self.publish(ENV.CALLS_NATS_EVENT.clone(), serde_json::to_string(&i)?.into()).await?;
        Ok(())
    }
    async fn send_to_user(
        &self, 
        event: CallEvent,
        user: String
    ) -> Result<()> {
        let i = InnerSignal{event, rcv: crate::types::Receiver::User(user)};
        self.publish(ENV.CALLS_NATS_EVENT.clone(), serde_json::to_string(&i)?.into()).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum User {
    Guest{guid: String, name: String},
    Logged{guid: String},
}
impl User {
    pub fn is_guest(&self) -> bool {
        match self {
            User::Guest{..} => true,
            _ => false
        }
    }
    pub fn guid(&self) -> String {
        match self {
            User::Guest{guid, ..} => guid.clone(),
            User::Logged{guid} => guid.clone()
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            User::Guest{guid, name} => write!(f, "{guid} (Guest {name})"),
            User::Logged{guid} => write!(f, "{guid} (User)")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomRecord {
    pub guid: String,
    pub name: String,
    pub owner: User,
    pub creator: User,
    pub created: i64,
    pub users: HashSet<User>,
    pub private: bool,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublicRoomRecord {
    guid: String,
    name: String,
    owner: User,
    creator: User,
    created: i64,
    users: HashSet<User>,
    private: bool,
    secure: bool,
}

impl RoomRecord {
    pub fn to_public(self) -> PublicRoomRecord {
        PublicRoomRecord{
            guid: self.guid,
            name: self.name,
            owner: self.owner,
            creator: self.creator,
            created: self.created,
            users: self.users,
            private: self.private,
            secure: self.password.is_some()
        }
    }
}

use crate::{types::{CallEvent, InnerSignal}, ENV};

fn room_to_parent_key(is_private: &bool) -> String{
    if is_private == &true {PRIVATE_ROOMS_KEY.to_string()} else {PUBLIC_ROOMS_KEY.to_string()}
}

fn room_to_key(room_guid: &str) -> String{
    format!("ROOM:{}", room_guid)
}

impl AppState{
    pub async fn new() -> Self {
        let c = async_nats::connect(format!("nats://{}:{}", ENV.NATS_URL, ENV.NATS_PORT)).await.expect("Can't connect to nats!");
        let inbox = c.new_inbox();
        let j = async_nats::jetstream::new(c);
        Self {
            signal_clients: Arc::new(RwLock::new(HashMap::new())),
            redis: RedisConn::default().await,
            jetstream: Arc::new(j),
            inbox
        }
    }
    pub async fn create_and_join_room(&self, room_name: String, private: bool, password: Option<String>, owner: User) -> RoomRecord {
        let guid = Uuid::new_v4().simple().to_string();
        let room = RoomRecord{
            guid: guid, 
            name: room_name, 
            creator: owner.clone(), 
            users: HashSet::from([owner.clone()]), 
            owner, 
            created: Utc::now().timestamp(), 
            private, 
            password: password
        };
        self.redis.hset(room_to_parent_key(&room.private), room_to_key(&room.guid), room.clone()).await.expect("Can't set room");
        self.jetstream.send_to_all(CallEvent::room_created(room.creator.clone(), room.clone().to_public())).await.expect("Can't send event");
        room
    }

    pub async fn delete_all_rooms(&self) {
        let _ = self.redis.del(PUBLIC_ROOMS_KEY).await;
        let _ = self.redis.del(PRIVATE_ROOMS_KEY).await;
    }

    pub async fn delete_room(&self, room_guid: String, is_private: &bool) -> Result<()> {
        self.redis.hdel(room_to_parent_key(is_private), room_to_key(&room_guid)).await;
        self.jetstream.send_to_all(CallEvent::room_deleted(room_guid.clone())).await?;
        Ok(())
    }

    pub async fn add_user_to_room(&self, mut room: RoomRecord, user: User) -> Result<()> {
        room.users.insert(user.clone());
        let guid = room.guid.clone();
        self.redis.hset(room_to_parent_key(&room.private), room_to_key(&room.guid), room).await;
        self.jetstream.send_to_all(CallEvent::room_join(user, guid)).await?;
        Ok(())
    }

    pub async fn rm_user_from_room(&self, room_guid: &String, is_private: &bool, user: &User) -> Result<()> {
        let pk = room_to_parent_key(is_private);
        let rk = room_to_key(room_guid);

        if let Some(mut r) = self.redis.hget::<RoomRecord>(&pk, &rk).await? {
            r.users.retain(|u| u.guid() != user.guid());
            if r.users.is_empty() {
                self.redis.hdel(&pk, &rk).await?;
                self.jetstream.send_to_all(CallEvent::room_deleted(room_guid.clone())).await?;
            } else {
                self.redis.hset(&pk, &rk, &r).await?;
            }
            self.jetstream.send_to_all(CallEvent::room_leave(user.clone(), room_guid.clone())).await?;
        }
        Ok(())
    }

    pub async fn get_rooms(&self) -> Result<HashMap<String, RoomRecord>>{
        let r : HashMap<String, RoomRecord> = self.redis.hgetall(PUBLIC_ROOMS_KEY).await?;
        Ok(r)
    }

    pub async fn get_all_rooms(&self) -> Result<(HashMap<String, RoomRecord>, HashMap<String, RoomRecord>)>{
        let r1 : HashMap<String, RoomRecord> = self.redis.hgetall(PUBLIC_ROOMS_KEY).await?;
        let r2 : HashMap<String, RoomRecord> = self.redis.hgetall(PRIVATE_ROOMS_KEY).await?;
        Ok((r1, r2))
    }

    pub async fn try_join_room(&self, room_guid: &String, user: &User, password: Option<String>) -> Result<RoomRecord, &str> {
        let Ok(room) = self.redis.hget::<RoomRecord>(room_to_parent_key(&false), room_to_key(room_guid)).await else {
            return Err("Server error")
        };
        let Some(room) = room else {return Err("Room not found")};
        if let Some(room_password) = &room.password {
            if &password.unwrap_or("".to_string()) != room_password {
                return Err("Wrong password")
            }
        }
        if self.add_user_to_room(room.clone(), user.clone()).await.is_err() {
            return Err("Server error")
        };
        if room.private {
            self.jetstream.send_to_room(CallEvent::RoomJoin { user: user.clone(), room_guid: room_guid.clone() }, room_guid.clone()).await
                .map_err(|e| error!("Error sending message to room: {}", e)).ok();
        } else {
            self.jetstream.send_to_all(CallEvent::RoomJoin { user: user.clone(), room_guid: room_guid.clone() }).await
                .map_err(|e| error!("Error sending message to room: {}", e)).ok();
        }
        Ok(room)
    }
}


