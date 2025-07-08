use std::{collections::HashSet};

use serde::{Deserialize, Serialize};

use super::{permissions::{PermissionInterface, PermissionPath, PermissionRuleNode}, users::Username};

pub type Groupname = String;

#[derive(Serialize, Deserialize)] 
#[derive(Clone, Debug)]
pub struct Group {
    pub(crate) name: Groupname,
    pub(crate) members: HashSet<Username>,

    pub(crate) permissions: PermissionRuleNode,

    pub(crate) parents: HashSet<Groupname>,
    pub(crate) children: HashSet<Groupname>,

    pub(crate) weight: i32
}

impl Group {
    pub fn new(name: Groupname, weight: i32) -> Self {
        Self {
            name,
            members: HashSet::new(),
            permissions: PermissionRuleNode::new(),
            parents: HashSet::new(),
            children: HashSet::new(),
            weight
        }
    }
    pub fn get_groupname(&self) -> &Groupname {&self.name}

    pub fn get_members(&self) -> &HashSet<Username> {&self.members}
    pub fn has_member(&self, member: &Username) -> bool {self.members.contains(member)}
    pub fn add_member(&mut self, member: Username) {self.members.insert(member);}
    pub fn add_members(&mut self, members: Vec<Username>) {for member in members {self.add_member(member);}}
    pub fn remove_member(&mut self, member: &Username) {self.members.remove(member);}
    pub fn remove_members(&mut self, members: Vec<Username>) {for member in members {self.remove_member(&member);}}

    pub fn get_parents(&self) -> &HashSet<Groupname> {&self.parents}
    pub fn has_parent(&self, parent: &Groupname) -> bool {self.parents.contains(parent)}
    pub fn add_parent(&mut self, parent: Groupname) {self.parents.insert(parent);}
    pub fn add_parents(&mut self, parents: Vec<Groupname>) {for parent in parents {self.add_parent(parent);}}
    pub fn remove_parent(&mut self, parent: &Groupname) {self.parents.remove(parent);}
    pub fn remove_parents(&mut self, parents: Vec<Groupname>) {for parent in parents {self.remove_parent(&parent);}}

    pub fn get_children(&self) -> &HashSet<Groupname> {&self.children}
    pub fn has_child(&self, group: &Groupname) -> bool {self.children.contains(group)}
    pub fn add_child(&mut self, group: Groupname) {self.children.insert(group);}
    pub fn remove_child(&mut self, group: &Groupname) {self.children.remove(group);}
    
    pub fn with_weight(self, weight: i32) -> Self {Self {weight, ..self} }
    pub fn set_weight(&mut self, weight: i32) { self.weight = weight }
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
    fn get_perm(&self, path: &PermissionPath) -> Option<bool> {self.permissions.get(path)}
    fn get_perms(&self) -> &PermissionRuleNode {&self.permissions}
    fn merge(&mut self, other: Self) {self.permissions.merge(other.permissions);}
}