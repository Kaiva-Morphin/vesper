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
    pub(crate) groups: HashSet<Groupname>,

    pub(crate) weight: usize
}

impl Group {
    pub fn new(name: Groupname, weight: usize) -> Self {
        Self {
            name,
            members: HashSet::new(),
            permissions: PermissionRuleNode::new(),
            parents: HashSet::new(),
            groups: HashSet::new(),
            weight
        }
    }
    pub fn get_groupname(&self) -> &Groupname {&self.name}

    pub fn get_members(&self) -> &HashSet<Username> {&self.members}
    pub fn has_member(&self, member: &Username) -> bool {self.members.contains(member)}
    pub fn add_member(&mut self, member: Username) {self.members.insert(member);}
    pub fn remove_member(&mut self, member: &Username) {self.members.remove(member);}

    pub fn get_parents(&self) -> &HashSet<Groupname> {&self.parents}
    pub fn has_parent(&self, parent: &Groupname) -> bool {self.parents.contains(parent)}
    pub fn add_parent(&mut self, parent: Groupname) {self.parents.insert(parent);}
    pub fn remove_parent(&mut self, parent: &Groupname) {self.parents.remove(parent);}

    pub fn get_groups(&self) -> &HashSet<Groupname> {&self.groups}
    pub fn has_group(&self, group: &Groupname) -> bool {self.groups.contains(group)}
    pub fn add_group(&mut self, group: Groupname) {self.groups.insert(group);}
    pub fn remove_group(&mut self, group: &Groupname) {self.groups.remove(group);}
    
    pub fn with_weight(self, weight: usize) -> Self {Self {weight, ..self} }
    pub fn set_weight(&mut self, weight: usize) { self.weight = weight }
}

impl PermissionInterface for Group {
    fn set_perm(&mut self, path: PermissionPath, enabled: bool) {self.permissions.set(path, enabled)}
    fn remove_perm(&mut self, path: &PermissionPath) {self.permissions.remove(path)}
    fn get_perm(&self, path: &PermissionPath) -> Option<bool> {self.permissions.get(path)}
    fn get_perms(&self) -> &PermissionRuleNode {&self.permissions}
    fn merge(&mut self, other: Self) {self.permissions.merge(other.permissions);}
}