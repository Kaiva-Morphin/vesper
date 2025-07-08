use std::{collections::HashMap, fmt::format};

use crate::prelude::{Groupname, PermissionPath, PermissionRule, Username};




#[derive(Debug)]
pub struct PermissionDelta {
    ops: Vec<PermissionOp>,
}

impl IntoIterator for PermissionDelta {
    type Item = PermissionOp;
    type IntoIter = std::vec::IntoIter<PermissionOp>;
    fn into_iter(self) -> Self::IntoIter {
        self.ops.into_iter()
    }
}

impl PermissionDelta {
    pub fn new() -> Self {
        Self {
            ops: vec![]
        }
    }
    pub fn push(&mut self, action: impl Into<PermissionOp>) {
        self.ops.push(action.into())
    }
    pub fn push_many(&mut self, actions: Vec<impl Into<PermissionOp>>){
        self.ops.extend(actions.into_iter().map(|v|v.into()));
    }

}

#[derive(Clone, Debug)]
pub enum PermissionOp {
    UserCreate(Username),
    UserRemove(Username),
    UserUpdatePerms(Username, Vec<PermissionRule>),
    UserRemovePerms(Username, Vec<PermissionPath>),

    GroupCreate{groupname: Groupname, weight: i32},
    GroupUpdate{groupname: Groupname, weight: i32},
    GroupRemove(Groupname),
    GroupUpdatePerms(Groupname, Vec<PermissionRule>),
    GroupRemovePerms(Groupname, Vec<PermissionPath>),
    GroupAddParentGroups(Groupname, Vec<Groupname>),
    GroupRemoveParentGroups(Groupname, Vec<Groupname>),
    GroupAddUsers(Groupname, Vec<Username>),
    GroupRemoveUsers(Groupname, Vec<Username>),
}
