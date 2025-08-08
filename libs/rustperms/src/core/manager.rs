use std::{collections::{hash_map::Entry, HashMap, HashSet, VecDeque}};
use crate::{api::actions::{RustpermsDelta, RustpermsOperation}, prelude::*};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bincode::serde::{decode_from_slice, encode_to_vec};
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
use tracing::info;

pub const GUEST_GROUP : &str = "guest";

impl AsyncManager {
    pub async fn check_perm(&self, user_uid: &UserUID, permission: &PermissionPath) -> Option<(bool, MatchType)> {
        let mut result_rule;
        let mut to_check: VecDeque<GroupUID> ;
        if user_uid == &"" {
            result_rule = (None, 0);
            to_check = VecDeque::from([GUEST_GROUP.to_string()]);
        } else {
            let users = self.users.read().await;
            let user= users.get(user_uid)?;
            result_rule = (user.get_perm(permission), RUSTPERMS_USER_WEIGHT);
            to_check = user.get_groups().iter().cloned().collect();
            drop(users);
        }
        info!("Checking permission {} for user {}", permission.format(), user_uid);
        info!("Groups to check: {:#?}", to_check);

        let mut checked: HashSet<GroupUID> = HashSet::new();
        let groups = self.groups.read().await;
        while let Some(group_uid) = to_check.pop_front() {
            if let Some(group) = groups.get(&group_uid) {
                'rule_update: {
                    let Some(allowed)= group.get_perm(permission) else {break 'rule_update};
                    let w = group.get_weight();
                    match result_rule.0 {
                        Some(result_allowed) => {
                            if w > result_rule.1 {
                                result_rule = (Some(allowed), w);
                            } else if w == result_rule.1 {
                                if result_allowed.1 < allowed.1 {
                                    // wildcard < any < exact
                                    // we pick highest rule
                                    result_rule = (Some(allowed), w);
                                } else if result_allowed.1 == allowed.1 {
                                    // false > true
                                    result_rule = (Some((allowed.0 && result_allowed.0, allowed.1)), w);
                                }
                            }
                        }
                        None => result_rule = (Some(allowed), w),
                    }
                }
                info!("Checked group {}", group_uid);
                info!("State: {:#?}", result_rule);
                info!("Parents: {:#?}", group.get_parents());
                for parent in group.get_parents() {
                    if !checked.contains::<GroupUID>(parent) {
                        to_check.push_back(parent.clone());
                    }
                }
            }
            checked.insert(group_uid);
        }
        result_rule.0
    }

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
            RustpermsOperation::GroupAddGroupsToInherit(g, gs) => {
                for gp in gs.iter() {
                    let Some(gr) = groups.get_mut(gp) else {continue};
                    gr.add_child(g.clone());
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.add_parents(gs);
                true
            },
            RustpermsOperation::GroupAddDependentGroups(g, gs) => {
                for gc in gs.iter() {
                    let Some(gr) = groups.get_mut(gc) else {continue};
                    gr.add_parent(g.clone());
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.add_children(gs);
                true
            },
            RustpermsOperation::GroupRemoveToInherit(g, gs) => {
                for gp in gs.iter() {
                    let Some(gr) = groups.get_mut(gp) else {continue};
                    gr.remove_child(&g);
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.remove_parents(gs);
                true
            },
            RustpermsOperation::GroupRemoveDependentGroups(g, gs) => {
                for gc in gs.iter() {
                    let Some(gr) = groups.get_mut(gc) else {continue};
                    gr.remove_parent(&g);
                }
                let Some(g) = groups.get_mut(&g) else {return false};
                g.remove_children(&gs);
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


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    fn path(s: &str) -> PermissionPath {
        PermissionPath::from_str(s)
    }

    fn rule(p: &str, allow: bool) -> PermissionRule {
        (path(p), allow)
    }

    #[tokio::test]
    async fn test_direct_user_permission() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("user1".into()),
            RustpermsOperation::UserUpdatePerms("user1".into(), vec![rule("a.b", true)]),
        ].into()).await;

        let result = manager.check_perm(&"user1".into(), &path("a.b")).await;
        assert_eq!(result, Some((true, MatchType::Exact)));
    }

    #[tokio::test]
    async fn test_group_permission() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("user1".into()),
            RustpermsOperation::GroupCreate { group_uid: "admin".into(), weight: 100 },
            RustpermsOperation::GroupUpdatePerms("admin".into(), vec![rule("a.b", true)]),
            RustpermsOperation::GroupAddUsers("admin".into(), vec!["user1".into()]),
        ].into()).await;

        let result = manager.check_perm(&"user1".into(), &path("a.b")).await;
        assert_eq!(result, Some((true, MatchType::Exact)));
    }

    #[tokio::test]
    async fn test_conflict_weight_resolution() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("user1".into()),
            RustpermsOperation::GroupCreate { group_uid: "low".into(), weight: 10 },
            RustpermsOperation::GroupCreate { group_uid: "high".into(), weight: 200 },
            RustpermsOperation::GroupUpdatePerms("low".into(), vec![rule("a.b", true)]),
            RustpermsOperation::GroupUpdatePerms("high".into(), vec![rule("a.b", false)]),
            RustpermsOperation::GroupAddUsers("low".into(), vec!["user1".into()]),
            RustpermsOperation::GroupAddUsers("high".into(), vec!["user1".into()]),
        ].into()).await;

        let result = manager.check_perm(&"user1".into(), &path("a.b")).await;
        assert_eq!(result, Some((false, MatchType::Exact)));
    }

    #[tokio::test]
    async fn test_group_inheritance() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("user1".into()),
            RustpermsOperation::GroupCreate { group_uid: "base".into(), weight: 10 },
            RustpermsOperation::GroupCreate { group_uid: "child".into(), weight: 15 },
            RustpermsOperation::GroupUpdatePerms("base".into(), vec![rule("a.b", true)]),
            RustpermsOperation::GroupAddGroupsToInherit("child".into(), vec!["base".into()]),
            RustpermsOperation::GroupAddUsers("child".into(), vec!["user1".into()]),
        ].into()).await;

        let result = manager.check_perm(&"user1".into(), &path("a.b")).await;
        assert_eq!(result, Some((true, MatchType::Exact)));
    }

    #[tokio::test]
    async fn test_user_not_found() {
        let manager = AsyncManager::default();
        let result = manager.check_perm(&"ghost".into(), &path("a.b")).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn user_vs_group_weight_conflict() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("u".into()),
            RustpermsOperation::UserUpdatePerms("u".into(), vec![(PermissionPath::from_str("a.b"), true)]),
            RustpermsOperation::GroupCreate { group_uid: "g".into(), weight: 2000 },
            RustpermsOperation::GroupUpdatePerms("g".into(), vec![(PermissionPath::from_str("a.b"), false)]),
            RustpermsOperation::GroupAddUsers("g".into(), vec!["u".into()]),
        ].into()).await;

        let res = manager.check_perm(&"u".into(), &path("a.b")).await;
        assert_eq!(res, Some((false, MatchType::Exact)));
    }

    #[tokio::test]
    async fn match_type_resolution_equal_weight() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("u".into()),
            RustpermsOperation::GroupCreate { group_uid: "g1".into(), weight: 100 },
            RustpermsOperation::GroupCreate { group_uid: "g2".into(), weight: 100 },
            RustpermsOperation::GroupUpdatePerms("g1".into(), vec![(path("a.*"), true)]),
            RustpermsOperation::GroupUpdatePerms("g2".into(), vec![(path("a.b"), false)]),
            RustpermsOperation::GroupAddUsers("g1".into(), vec!["u".into()]),
            RustpermsOperation::GroupAddUsers("g2".into(), vec!["u".into()]),
        ].into()).await;

        let res = manager.check_perm(&"u".into(), &path("a.b")).await;
        assert_eq!(res, Some((false, MatchType::Exact))); // Exact > Wildcard
    }

    #[tokio::test]
    async fn match_type_merge_equal_weight_and_type() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("u".into()),
            RustpermsOperation::GroupCreate { group_uid: "g1".into(), weight: 100 },
            RustpermsOperation::GroupCreate { group_uid: "g2".into(), weight: 100 },
            RustpermsOperation::GroupUpdatePerms("g1".into(), vec![(path("a.b"), false)]),
            RustpermsOperation::GroupUpdatePerms("g2".into(), vec![(path("a.b"), true)]),
            RustpermsOperation::GroupAddUsers("g1".into(), vec!["u".into()]),
            RustpermsOperation::GroupAddUsers("g2".into(), vec!["u".into()]),
        ].into()).await;

        let res = manager.check_perm(&"u".into(), &path("a.b")).await;
        assert_eq!(res, Some((false, MatchType::Exact))); // false && true = false
    }

    #[tokio::test]
    async fn inherited_nested_group() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("u".into()),
            RustpermsOperation::GroupCreate { group_uid: "g1".into(), weight: 10 },
            RustpermsOperation::GroupCreate { group_uid: "g2".into(), weight: 20 },
            RustpermsOperation::GroupUpdatePerms("g1".into(), vec![(path("x.y"), true)]),
            RustpermsOperation::GroupAddGroupsToInherit("g2".into(), vec!["g1".into()]),
            RustpermsOperation::GroupAddUsers("g2".into(), vec!["u".into()]),
        ].into()).await;

        let res = manager.check_perm(&"u".into(), &path("x.y")).await;
        assert_eq!(res, Some((true, MatchType::Exact)));
    }

    #[tokio::test]
    async fn circular_group_check() {
        let manager = AsyncManager::default();
        manager.apply(vec![
            RustpermsOperation::UserCreate("u".into()),
            RustpermsOperation::GroupCreate { group_uid: "g1".into(), weight: 50 },
            RustpermsOperation::GroupCreate { group_uid: "g2".into(), weight: 60 },
            RustpermsOperation::GroupUpdatePerms("g1".into(), vec![(path("a.b"), true)]),
            RustpermsOperation::GroupAddGroupsToInherit("g1".into(), vec!["g2".into()]),
            RustpermsOperation::GroupAddGroupsToInherit("g2".into(), vec!["g1".into()]),
            RustpermsOperation::GroupAddUsers("g1".into(), vec!["u".into()]),
        ].into()).await;

        let res = manager.check_perm(&"u".into(), &path("a.b")).await;
        assert_eq!(res, Some((true, MatchType::Exact)));
    }
}
