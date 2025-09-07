use serde::{Serialize, Deserialize};

use crate::state::{PublicRoomRecord, RoomRecord, User};

#[derive(Serialize,Deserialize,Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientRequests {
    Leave,
    Join {room_guid: String, password: Option<String>},
    Create {
        name: String,
        private: Option<bool>,
        password: Option<String>,
    },
    Message {msg: String},
    Pong,
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InnerSignal {
    pub rcv: Receiver,
    pub event: CallEvent
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Receiver {
    User(String),
    Room(String),
    All
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CallEvent {
    RoomJoin {user: User, room_guid: String},
    RoomLeave {user: User, room_guid: String},
    RoomCreated {user: User, room: PublicRoomRecord},
    RoomDeleted {room_guid: String},
    Message {msg: String},
    Ping,
    Error {msg: String}
}

impl CallEvent {
    pub fn room_join(user: User, room_guid: String) -> CallEvent {CallEvent::RoomJoin{user, room_guid}}
    pub fn room_leave(user: User, room_guid: String) -> CallEvent {CallEvent::RoomLeave{user, room_guid}}
    pub fn room_created(user: User, room: PublicRoomRecord) -> CallEvent {CallEvent::RoomCreated{user, room}}
    pub fn room_deleted(room_guid: String) -> CallEvent {CallEvent::RoomDeleted{room_guid}}
    pub fn message(msg: String) -> CallEvent {CallEvent::Message{msg}}
}