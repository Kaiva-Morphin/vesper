use rustperms::prelude::{IntoPermPath, RustpermsOperation};
use uuid::Uuid;
use shared::utils::IntoKey;

use crate::{groups::{DEFAULT_GROUP, AUTHED_GROUP}, rule, user::profile::{miniprofile_edit_perm_postfix, profile_edit_perm_postfix}};

pub mod profile;

pub fn create_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    vec![
        RustpermsOperation::UserCreate(key.clone()),
        RustpermsOperation::GroupAddUsers(DEFAULT_GROUP.to_string(), vec![key.clone()]),
        RustpermsOperation::GroupAddUsers(AUTHED_GROUP.to_string(), vec![key]),
    ]
}

pub fn delete_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    vec![
        RustpermsOperation::UserRemove(key.clone()),
        RustpermsOperation::GroupRemoveUsers(DEFAULT_GROUP.to_string(), vec![key.clone()]),
        RustpermsOperation::GroupRemoveUsers(AUTHED_GROUP.to_string(), vec![key]),
    ]
}



pub fn grant_default_for_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    let p = vec![
        (profile_edit_perm_postfix(&key, "*").into_perm(), true),
        (miniprofile_edit_perm_postfix(&key, "*").into_perm(), true),
    ];
    vec![
        RustpermsOperation::UserUpdatePerms(key, p)
    ]
}

// pub fn revoke_default_for_user(guid: &Uuid) -> Vec<RustpermsOperation> {
//     let key = guid.into_key();
//     let p = vec![
//         (profile_edit_perm(&key).into_perm()),
//         (profile_edit_perm_postfix(&key, "*").into_perm()),
//     ];
//     vec![
//         RustpermsOperation::UserRemovePerms(key, p)
//     ]
// }