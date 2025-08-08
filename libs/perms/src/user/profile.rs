use rustperms::{api::util::sharded_group_update_perms, prelude::*};
use uuid::Uuid;
use crate::{groups::*, user::IntoKey};



macro_rules! rule {
    ($const_name:ident, $value:expr) => {
        pub const $const_name: &str = $value;

        paste::paste! {
            pub fn [<$const_name:lower>](key: &str) -> String {
                format!("{value}.{key}", value = $value)
            }
        }
    };
}


rule!(PROFILE_EDIT_PERM, "user.profile.edit");
rule!(PROFILE_VIEW_PERM, "user.profile.view");


pub fn grant_default() -> Vec<RustpermsOperation> {
    sharded_group_update_perms(DEFAULT_GROUP.to_string(), DEFAULT_SHARDING, vec![
        ("*", (profile_view_perm("*").into_perm(), true)),
    ])
}



pub fn grant_default_for_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    let p = vec![(profile_edit_perm(&key).into_perm(), true)];
    vec![
        RustpermsOperation::UserUpdatePerms(key, p)
    ]
}

pub fn revoke_default_for_user(guid: &Uuid) -> Vec<RustpermsOperation> {
    let key = guid.into_key();
    let p = vec![(profile_edit_perm(&key).into_perm())];
    vec![
        RustpermsOperation::UserRemovePerms(key, p)
    ]
}