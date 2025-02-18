use rand::random;

use super::records::{Room, RoomId, UserId};
use crate::calls::records::RoomIdExt;

#[derive(serde::Deserialize)]
pub struct CreateRoom {
    pub name: String,
    pub description: Option<String>,
    pub password: Option<String>,
    pub public: bool,
    pub max_users: u8,
}
impl CreateRoom {
    pub fn as_room(&self, owner: UserId, public_owner: String) -> Option<Room> {
        if self.max_users < 2 {
            return None;
        }
        Some(
            Room {
                id: RoomId::get_public(self.public),
                owner,
                public_owner,
                max_users: self.max_users,
                users: vec![],
                name: self.name.clone(),
                description: self.description.clone(),
                password: self.password.clone(),
                empty_since: 0,
            }
        )
    }
}













