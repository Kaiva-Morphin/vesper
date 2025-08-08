use rustperms::prelude::RustpermsOperation;
use uuid::Uuid;
use shared::utils::IntoKey;

use crate::groups::{DEFAULT_GROUP, LOGGED_GROUP};

pub mod profile;


pub fn create_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    vec![
        RustpermsOperation::UserCreate(key.clone()),
        RustpermsOperation::GroupAddUsers(DEFAULT_GROUP.to_string(), vec![key.clone()]),
        RustpermsOperation::GroupAddUsers(LOGGED_GROUP.to_string(), vec![key]),

    ]
}

pub fn delete_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    vec![
        RustpermsOperation::UserRemove(key.clone()),
        RustpermsOperation::GroupRemoveUsers(DEFAULT_GROUP.to_string(), vec![key.clone()]),
        RustpermsOperation::GroupRemoveUsers(LOGGED_GROUP.to_string(), vec![key]),
    ]
}