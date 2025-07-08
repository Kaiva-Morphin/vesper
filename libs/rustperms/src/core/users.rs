use std::{borrow::Borrow, collections::HashSet};

use serde::{Deserialize, Serialize};


use super::{groups::Groupname, permissions::{PermissionInterface, PermissionPath, PermissionRuleNode}};

pub type Username = String;

#[derive(Serialize, Deserialize)] 
#[derive(Clone, Debug)]
pub struct User {
    pub username: Username,
    pub groups: HashSet<Groupname>,
    pub permissions: PermissionRuleNode,
}

impl User {
    pub fn new(username: Username) -> Self {
        Self {
            username,
            groups: HashSet::new(),
            permissions: PermissionRuleNode::new(),
        }
    }
    pub fn get_username(&self) -> &Username {&self.username}

    pub fn get_groups(&self) -> &HashSet<Groupname> {&self.groups}
    pub fn has_group(&self, group: &Groupname) -> bool {self.groups.contains(group)}
    pub fn add_group(&mut self, group: Groupname) {self.groups.insert(group);}
    pub fn remove_group(&mut self, group: &Groupname) {self.groups.remove(group);}

    pub fn get_perms(&self) -> &PermissionRuleNode {&self.permissions}
    
}

impl PermissionInterface for User {
    fn set_perm(&mut self, path: PermissionPath, enabled: bool) {self.permissions.set(path, enabled)}
    fn set_perms(&mut self, perms: Vec<super::prelude::PermissionRule>) {
        for (perm, enabled) in perms {
            self.set_perm(perm, enabled);
        }
    }
    fn remove_perm(&mut self, path: &PermissionPath) {self.permissions.remove(path)}
    fn remove_perms(&mut self, perms: Vec<PermissionPath>) {
        for path in perms {
            self.remove_perm(&path);
        }
    }
    fn get_perm(&self, path: &PermissionPath) -> Option<bool> {self.permissions.get(path)}
    fn get_perms(&self) -> &PermissionRuleNode {&self.permissions}
    fn merge(&mut self, other: Self) {self.permissions.merge(other.permissions);}
}