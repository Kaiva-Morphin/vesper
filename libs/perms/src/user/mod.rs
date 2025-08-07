use rustperms::prelude::RustpermsOperation;
use uuid::Uuid;

pub mod profile;

pub trait IntoKey {
    fn into_key(&self) -> String;
}

impl IntoKey for Uuid {
    fn into_key(&self) -> String {
        self.simple().to_string()
    }
}


pub fn create_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    vec![RustpermsOperation::UserCreate(key)]
}

pub fn delete_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    vec![RustpermsOperation::UserRemove(key)]
}