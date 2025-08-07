use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};

use crate::prelude::{GroupUID, PermissionPath, PermissionRule, UserUID};




#[derive(Debug, Clone)]
pub struct RustpermsDelta {
    ops: Vec<RustpermsOperation>,
}

impl IntoIterator for RustpermsDelta {
    type Item = RustpermsOperation;
    type IntoIter = std::vec::IntoIter<RustpermsOperation>;
    fn into_iter(self) -> Self::IntoIter {
        self.ops.into_iter()
    }
}

impl Default for RustpermsDelta {
    fn default() -> Self {
        Self::new()
    }
}

impl RustpermsDelta {
    pub fn new() -> Self {
        Self {
            ops: vec![]
        }
    }
    pub fn push(&mut self, action: impl Into<RustpermsOperation>) {
        self.ops.push(action.into())
    }
    pub fn push_many(&mut self, actions: Vec<impl Into<RustpermsOperation>>){
        self.ops.extend(actions.into_iter().map(|v|v.into()));
    }
    pub fn serialize_to_string(self) -> anyhow::Result<String> {
        let e = encode_to_vec(self.ops, bincode::config::standard())?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(e))
    }
    pub fn deserialize_from_string(serialized: &str) -> anyhow::Result<Self>  {
        let (ops, _) : (Vec<RustpermsOperation>, _) = decode_from_slice(&BASE64_URL_SAFE_NO_PAD.decode(serialized)?, bincode::config::standard())?;
        Ok(Self{ops})
    }
}

impl From<Vec<RustpermsOperation>> for RustpermsDelta {
    fn from(value: Vec<RustpermsOperation>) -> Self {
        Self {
            ops: value
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RustpermsOperation {
    UserCreate(UserUID),
    UserRemove(UserUID),
    UserUpdatePerms(UserUID, Vec<PermissionRule>),
    UserRemovePerms(UserUID, Vec<PermissionPath>),

    GroupCreate{group_uid: GroupUID, weight: i32},
    GroupUpdate{group_uid: GroupUID, weight: i32},
    GroupRemove(GroupUID),
    GroupUpdatePerms(GroupUID, Vec<PermissionRule>),
    GroupRemovePerms(GroupUID, Vec<PermissionPath>),
    GroupAddParentGroups(GroupUID, Vec<GroupUID>),
    GroupAddChildrenGroups(GroupUID, Vec<GroupUID>),
    GroupRemoveParentGroups(GroupUID, Vec<GroupUID>),
    GroupRemoveChildrenGroups(GroupUID, Vec<GroupUID>),
    GroupAddUsers(GroupUID, Vec<UserUID>),
    GroupRemoveUsers(GroupUID, Vec<UserUID>),
}
