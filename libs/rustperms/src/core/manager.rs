use std::{collections::{hash_map::Entry, HashMap}, default};
use crate::{api::actions::{RustpermsDelta, RustpermsOperation}, core::groups, prelude::*};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bincode::serde::{decode_from_slice, encode_into_slice, encode_to_vec};
use ::tokio::sync::RwLock;



#[derive(Debug)]
pub struct AsyncManager {
    pub users: RwLock<HashMap<UserUID, User>>,
    pub groups: RwLock<HashMap<GroupUID, Group>>,
}

impl AsyncManager {
    pub async fn eq(&self, other: &Self) -> bool {
        *self.users.read().await == *other.users.read().await &&
        *self.groups.read().await == *other.groups.read().await
    }
}

impl Default for AsyncManager {
    fn default() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
        }
    }
}

impl From<RustpermsDelta> for AsyncManager {
    fn from(actions: RustpermsDelta) -> Self {
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
        let (users, _): (HashMap<UserUID, User>, _)  = decode_from_slice(&BASE64_URL_SAFE_NO_PAD.decode(serialized_users)?, bincode::config::standard())?;
        let (groups, _): (HashMap<GroupUID, Group>, _)  = decode_from_slice(&BASE64_URL_SAFE_NO_PAD.decode(serialized_groups)?, bincode::config::standard())?;
        Ok(Self {
            users: RwLock::new(users),
            groups: RwLock::new(groups)
        })
    }

    pub fn apply_action(users: &mut HashMap<UserUID, User>, groups: &mut HashMap<GroupUID, Group>, action: RustpermsOperation) -> bool {
        match action {
            RustpermsOperation::UserCreate(u) => {
                let e@ Entry::Vacant(_) = users.entry(u.clone()) else {return false};
                e.or_insert_with(|| User::new(u));
                true
            }
            RustpermsOperation::UserRemove(u) => {
                let Some(user) = users.remove(&u) else {return false;};
                for g in user.groups {
                    groups.entry(g).and_modify(|g| g.remove_member(&u));
                }
                true
            }
            RustpermsOperation::UserUpdatePerms(u, p) => {
                let Some(u) = users.get_mut(&u) else {return false};
                u.set_perms(p);
                true
            }
            RustpermsOperation::UserRemovePerms(u, p) => {
                let Some(u) = users.get_mut(&u) else {return false};
                u.remove_perms(p);
                true
            }
            RustpermsOperation::GroupCreate{group_uid: g, weight: w} => {
                let None = groups.get(&g) else {return false};
                groups.insert(g.clone(), Group::new(g, w));
                true
            },
            RustpermsOperation::GroupUpdate { group_uid: g, weight: w } => {
                let Some(g) = groups.get_mut(&g) else {return false};
                g.set_weight(w);
                true
            },
            RustpermsOperation::GroupRemove(g) => {
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
            RustpermsOperation::GroupUpdatePerms(g, p) => {
                let Some(g) = groups.get_mut(&g) else {return false};
                g.set_perms(p);
                true
            },
            RustpermsOperation::GroupRemovePerms(g, ps) => {
                let Some(g) = groups.get_mut(&g) else {return false};
                g.remove_perms(ps);
                true
            }
            RustpermsOperation::GroupAddParentGroups(g, gs) => {
                for gp in gs.iter() {
                    let Some(gr) = groups.get_mut(gp) else {continue};
                    gr.add_child(g.clone());
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.add_parents(gs);
                true
            },
            RustpermsOperation::GroupRemoveParentGroups(g, gs) => {
                for gp in gs.iter() {
                    let Some(gr) = groups.get_mut(gp) else {continue};
                    gr.remove_child(&g);
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.remove_parents(gs);
                true
            },
            RustpermsOperation::GroupAddUsers(g, us) => {
                let Some(group) = groups.get_mut(&g) else {return false};
                for user in us {
                    let Some(u) = users.get_mut(&user) else {continue};
                    group.add_member(user);
                    u.add_group(g.clone());
                }
                true
            },
            RustpermsOperation::GroupRemoveUsers(g, us) => {
                for user in &us {
                    let Some(u) = users.get_mut(user) else {continue};
                    u.remove_group(&g);
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.remove_members(us);
                true
            },
        }
    }

    pub async fn apply(&self, actions: RustpermsDelta) {
        let mut users = self.users.write().await;
        let mut groups = self.groups.write().await;
        for action in actions.into_iter() {
            Self::apply_action(&mut users, &mut groups, action);
        }
    }
}
