
use rustperms::{api::util::{sharded_group_crate, sharded_group_update_perms}, prelude::*};
use rustperms_nodes::proto::{rustperms_master_proto_client::RustpermsMasterProtoClient, WriteRequest};

use crate::{rule, user::profile::profile_view_perm};

pub const AUTHED_GROUP : &str = "authed";
pub const DEFAULT_GROUP : &str = "default";

pub const AUTHED_SHARDING : usize = 32;
pub const GUEST_SHARDING : usize = 32;
pub const DEFAULT_SHARDING : usize = 32;

pub fn init_default() -> Vec<RustpermsOperation> {
    let mut ops = sharded_group_crate(AUTHED_GROUP.to_string(), 10, AUTHED_SHARDING);
    ops.extend(sharded_group_crate(GUEST_GROUP.to_string(), 5, GUEST_SHARDING).into_iter());
    ops.extend(sharded_group_crate(DEFAULT_GROUP.to_string(), 0, DEFAULT_SHARDING).into_iter());
    ops.extend(
        vec![
            RustpermsOperation::GroupAddDependentGroups(DEFAULT_GROUP.to_string(), vec![AUTHED_GROUP.to_string(), GUEST_GROUP.to_string()]),
        ]
    );
    ops
}

rule!(UPLOAD_TO_STORE_PERM, "store.upload");

rule!(CALLS_PERM, "calls");

pub fn fill_with_defaults() -> Vec<RustpermsOperation> {
    let mut ops = sharded_group_update_perms(DEFAULT_GROUP.to_string(), DEFAULT_SHARDING, vec![
        ("profile", (profile_view_perm("*").into_perm(), true)),
        ("calls", (calls_perm("*").into_perm(), true)),
        ("calls", (calls_perm("view.hidden").into_perm(), false)),
    ]);
    ops.extend(
        sharded_group_update_perms(AUTHED_GROUP.to_string(), AUTHED_SHARDING, vec![
            ("*", (upload_to_store_perm("*").into_perm(), true)),
        ])
    );
    ops
}
