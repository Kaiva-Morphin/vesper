use std::{collections::HashSet};

use serde::{Deserialize, Serialize};

use crate::prelude::MatchType;

use super::{permissions::{PermissionInterface, PermissionPath, PermissionRuleNode}, users::UserUID};

pub type GroupUID = String;

#[derive(Serialize, Deserialize)] 
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq)]
pub struct Group {
    pub(crate) name: GroupUID,
    pub(crate) members: HashSet<UserUID>,

    pub(crate) permissions: PermissionRuleNode,

    pub(crate) parents: HashSet<GroupUID>,
    pub(crate) children: HashSet<GroupUID>,

    pub(crate) weight: i32
}

impl Group {
    pub fn new(name: GroupUID, weight: i32) -> Self {
        Self {
            name,
            members: HashSet::new(),
            permissions: PermissionRuleNode::new(),
            parents: HashSet::new(),
            children: HashSet::new(),
            weight
        }
    }
    pub fn get_group_uid(&self) -> &GroupUID {&self.name}

    pub fn get_members(&self) -> &HashSet<UserUID> {&self.members}
    pub fn has_member(&self, member: &UserUID) -> bool {self.members.contains(member)}
    pub fn add_member(&mut self, member: UserUID) {self.members.insert(member);}
    pub fn add_members(&mut self, members: Vec<UserUID>) {for member in members {self.add_member(member);}}
    pub fn remove_member(&mut self, member: &UserUID) {self.members.remove(member);}
    pub fn remove_members(&mut self, members: Vec<UserUID>) {for member in members {self.remove_member(&member);}}

    pub fn get_parents(&self) -> &HashSet<GroupUID> {&self.parents}
    pub fn has_parent(&self, parent: &GroupUID) -> bool {self.parents.contains(parent)}
    pub fn add_parent(&mut self, parent: GroupUID) {self.parents.insert(parent);}
    pub fn add_parents(&mut self, parents: Vec<GroupUID>) {for parent in parents {self.add_parent(parent);}}
    pub fn remove_parent(&mut self, parent: &GroupUID) {self.parents.remove(parent);}
    pub fn remove_parents(&mut self, parents: Vec<GroupUID>) {for parent in parents {self.remove_parent(&parent);}}

    pub fn get_children(&self) -> &HashSet<GroupUID> {&self.children}
    pub fn has_child(&self, group: &GroupUID) -> bool {self.children.contains(group)}
    pub fn add_child(&mut self, group: GroupUID) {self.children.insert(group);}
    pub fn add_children(&mut self, groups: Vec<GroupUID>) {self.children.extend(groups)}
    pub fn remove_child(&mut self, group: &GroupUID) {self.children.remove(group);}
    pub fn remove_children(&mut self, groups: &Vec<GroupUID>) {for group in groups {self.children.remove(group);}}
    
    pub fn with_weight(self, weight: i32) -> Self {Self {weight, ..self} }
    pub fn set_weight(&mut self, weight: i32) { self.weight = weight }
    pub fn get_weight(&self) -> i32 {self.weight}
}

impl PermissionInterface for Group {
    fn set_perm(&mut self, path: PermissionPath, enabled: bool) {self.permissions.set(path, enabled)}
    fn set_perms(&mut self, perms: Vec<super::prelude::PermissionRule>) {
        for (p, e) in perms {self.set_perm(p, e);}
    }
    fn remove_perm(&mut self, path: &PermissionPath) {self.permissions.remove(path)}
    fn remove_perms(&mut self, perms: Vec<PermissionPath>) {
        for path in perms {self.remove_perm(&path)}
    }
    fn get_perm(&self, path: &PermissionPath) -> Option<(bool, MatchType)> {self.permissions.get(path)}
    fn get_perms(&self) -> &PermissionRuleNode {&self.permissions}
    fn merge(&mut self, other: Self) {self.permissions.merge(other.permissions);}
}