use core::hash;
use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use xxhash_rust::xxh3::xxh3_64;

use crate::prelude::*;

// "proxy" for high-loaded groups

pub fn group_shard(group: &GroupUID, shard: usize) -> GroupUID {
    format!("{}.{}", group, shard)
}

pub fn group_to_shards(group: &GroupUID, shards: usize) -> Vec<GroupUID> {
    let mut ids = Vec::with_capacity(shards + 1);
    for i in 0..shards {
        ids.push(group_shard(group, i));
    }
    ids
}

pub fn group_to_sharded(group: GroupUID, shards: usize) -> Vec<GroupUID> {
    let mut ids = Vec::with_capacity(shards + 1);
    for i in 0..shards {
        ids.push(group_shard(&group, i));
    }
    ids.push(group);
    ids
}


pub fn key_to_shard_suffix(key: &str, shards: usize) -> usize {
    let hash = xxh3_64(key.as_bytes());
    (hash as usize) % shards
}

pub fn sharded_group_crate(group: GroupUID, weight: i32, shards: usize) -> Vec<RustpermsOperation> {
    let g = group_to_shards(&group, shards);
    let mut gsc = g.clone();
    gsc.push(group.clone());
    gsc.into_iter()
        .map(|n| RustpermsOperation::GroupCreate{group_uid: n, weight})
        .chain([RustpermsOperation::GroupAddChildrenGroups(group, g)].into_iter())
        .collect::<Vec<RustpermsOperation>>()
}

pub fn sharded_group_remove(group: GroupUID, shards: usize) -> Vec<RustpermsOperation> {
    group_to_sharded(group, shards).into_iter().map(|n| RustpermsOperation::GroupRemove(n)).collect()
}

pub fn sharded_group_update(group: GroupUID, shards: usize, weight: i32) -> Vec<RustpermsOperation> {
    group_to_sharded(group, shards).into_iter().map(|n| RustpermsOperation::GroupUpdate{group_uid: n, weight}).collect()
}

pub fn sharded_group_remove_perms(group: GroupUID, shards: usize, perms: Vec<(&str, PermissionPath)>) -> Vec<RustpermsOperation> {
    let mut map : HashMap<usize, Vec<PermissionPath>> = HashMap::new();
    for (key, perm) in perms {
        let shard = key_to_shard_suffix(key, shards);
        map.entry(shard).or_insert_with(Vec::new).push(perm);
    }
    map.into_iter().map(|(k, v)| RustpermsOperation::GroupRemovePerms(group_shard(&group, k), v)).collect()
}

pub fn sharded_group_update_perms(group: GroupUID, shards: usize, perms: Vec<(&str, PermissionRule)>) -> Vec<RustpermsOperation> {
    let mut map : HashMap<usize, Vec<PermissionRule>> = HashMap::new();
    for (key, perm) in perms {
        let shard = key_to_shard_suffix(key, shards);
        map.entry(shard).or_insert_with(Vec::new).push(perm);
    }
    map.into_iter().map(|(k, v)| RustpermsOperation::GroupUpdatePerms(group_shard(&group, k), v)).collect()
}