
use rustperms::{api::util::sharded_group_crate, prelude::*};
use rustperms_nodes::proto::{rustperms_master_proto_client::RustpermsMasterProtoClient, WriteRequest};

pub const LOGGED_GROUP : &str = "logged";
pub const DEFAULT_GROUP : &str = "default";

pub const LOGGED_SHARDING : usize = 32;
pub const GUEST_SHARDING : usize = 32;
pub const DEFAULT_SHARDING : usize = 32;

pub fn init_default() -> Vec<RustpermsOperation> {
    let mut ops = sharded_group_crate(LOGGED_GROUP.to_string(), 10, LOGGED_SHARDING);
    ops.extend(sharded_group_crate(GUEST_GROUP.to_string(), 5, GUEST_SHARDING).into_iter());
    ops.extend(sharded_group_crate(DEFAULT_GROUP.to_string(), 0, DEFAULT_SHARDING).into_iter());
    ops.extend(
        vec![
            RustpermsOperation::GroupAddDependentGroups(DEFAULT_GROUP.to_string(), vec![LOGGED_GROUP.to_string(), GUEST_GROUP.to_string()]),
        ]
    );
    ops
}


