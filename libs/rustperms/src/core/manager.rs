use std::{collections::{hash_map::Entry, HashMap}, default};
use crate::{api::actions::{PermissionDelta, PermissionOp}, core::groups, prelude::*};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bincode::serde::{decode_from_slice, encode_into_slice, encode_to_vec};
use ::tokio::sync::RwLock;



#[derive(Debug)]
pub struct AsyncManager {
    pub users: RwLock<HashMap<Username, User>>,
    pub groups: RwLock<HashMap<Groupname, Group>>,
}

impl Default for AsyncManager {
    fn default() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
        }
    }
}

impl From<PermissionDelta> for AsyncManager {
    fn from(actions: PermissionDelta) -> Self {
        let mut users = HashMap::new();
        let mut groups = HashMap::new();
        for action in actions.into_iter() {
            Self::apply_action(&mut users, &mut groups, action);
        }
        Self {users: RwLock::new(users), groups: RwLock::new(groups)}
    }
}

use anyhow::Result;

impl AsyncManager {
    pub async fn users_to_string(&self) -> Result<String> {
        let u = self.users.read().await;
        let users = u.clone();
        drop(u);
        let u = encode_to_vec(users, bincode::config::standard())?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(u))
    }
    pub async fn groups_to_string(&self) -> Result<String> {
        let g = self.groups.read().await;
        let groups = g.clone();
        drop(g);
        let g = encode_to_vec(groups, bincode::config::standard())?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(g))
    }
    pub fn from_serialized_string(serialized_users: &str, serialized_groups: &str) -> Result<Self> {
        let (users, _): (HashMap<Username, User>, _)  = decode_from_slice(&BASE64_URL_SAFE_NO_PAD.decode(serialized_users)?, bincode::config::standard())?;
        let (groups, _): (HashMap<Groupname, Group>, _)  = decode_from_slice(&BASE64_URL_SAFE_NO_PAD.decode(serialized_groups)?, bincode::config::standard())?;
        Ok(Self {
            users: RwLock::new(users),
            groups: RwLock::new(groups)
        })
    }

    pub fn apply_action(users: &mut HashMap<Username, User>, groups: &mut HashMap<Groupname, Group>, action: PermissionOp) -> bool {
        match action {
            PermissionOp::UserCreate(u) => {
                let e@ Entry::Vacant(_) = users.entry(u.clone()) else {return false};
                e.or_insert_with(|| User::new(u));
                true
            }
            PermissionOp::UserRemove(u) => {
                let Some(user) = users.remove(&u) else {return false;};
                for g in user.groups {
                    groups.entry(g).and_modify(|g| g.remove_member(&u));
                }
                true
            }
            PermissionOp::UserUpdatePerms(u, p) => {
                let e@ Entry::Occupied(_) = users.entry(u.clone()) else {return false};
                e.and_modify(|u| u.set_perms(p));
                true
            }
            PermissionOp::UserRemovePerms(u, p) => {
                let e @ Entry::Occupied(_) = users.entry(u.clone()) else {return false};
                e.and_modify(|u| u.remove_perms(p));
                true
            }
            PermissionOp::GroupCreate{groupname: g, weight: w} => {
                let e @ Entry::Vacant(_) = groups.entry(g.clone()) else {return false};
                e.or_insert_with(|| Group::new(g, w));
                true
            },
            PermissionOp::GroupUpdate { groupname: g, weight: w } => {
                let e @ Entry::Occupied(_) = groups.entry(g) else {return false};
                e.and_modify(|g| g.set_weight(w));
                true
            },
            PermissionOp::GroupRemove(g) => {
                let Some(group) = groups.remove(&g) else {return false;};
                for u in group.members {
                    users.entry(u).and_modify(|u| u.remove_group(&g));
                }
                for gc in group.children {
                    groups.entry(gc).and_modify(|gc| gc.remove_parent(&g));
                }
                for gp in group.parents {
                    groups.entry(gp).and_modify(|gp| gp.remove_child(&g));
                }
                true
            },
            PermissionOp::GroupUpdatePerms(g, p) => {
                let e@ Entry::Occupied(_) = groups.entry(g.clone()) else {return false};
                e.and_modify(|g| g.set_perms(p));
                true
            },
            PermissionOp::GroupRemovePerms(g, ps) => {
                let e @ Entry::Occupied(_) = groups.entry(g.clone()) else {return false};
                e.and_modify(|g| g.remove_perms(ps));
                true
            }
            PermissionOp::GroupAddParentGroups(g, gs) => {
                for gp in gs.iter() {
                    let Some(gr) = groups.get_mut(gp) else {continue};
                    gr.add_child(g.clone());
                }
                let e @ Entry::Occupied(_) = groups.entry(g.clone()) else {return false};
                e.and_modify(|g| g.add_parents(gs));
                true
            },
            PermissionOp::GroupRemoveParentGroups(g, gs) => {
                let e @ Entry::Occupied(_) = groups.entry(g.clone()) else {return false};
                e.and_modify(|g| g.remove_parents(gs));
                true
            },
            PermissionOp::GroupAddUsers(g, us) => {
                let Some(group) = groups.get_mut(&g) else {return false};
                for user in us {
                    let Some(u) = users.get_mut(&user) else {continue};
                    group.add_member(user);
                    u.add_group(g.clone());
                }
                true
            },
            PermissionOp::GroupRemoveUsers(g, us) => {
                let e @ Entry::Occupied(_) = groups.entry(g.clone()) else {return false};
                e.and_modify(|g| g.remove_members(us));
                true
            },
        }
    }

    pub async fn apply(&mut self, actions: PermissionDelta) {
        let mut users = self.users.write().await;
        let mut groups = self.groups.write().await;
        for action in actions.into_iter() {
            Self::apply_action(&mut users, &mut groups, action);
        }
    }
}
