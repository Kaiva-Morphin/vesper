use std::collections::HashMap;

use rustperms::{api::actions::RustpermsOperation, prelude::*};
use sqlx::prelude::*;

pub trait FromBatch<T> where Self: Sized {
    fn from_batch(batch: Vec<T>) -> Vec<Self>;
}

impl<T: Into<RustpermsOperation>> FromBatch<T> for RustpermsOperation {
    fn from_batch(batch: Vec<T>) -> Vec<RustpermsOperation> {
        batch.into_iter().map(|u| u.into()).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct UserModel {
    user_uid: String
}

impl From<UserModel> for RustpermsOperation {
    fn from(value: UserModel) -> Self {
        RustpermsOperation::UserCreate(value.user_uid)
    }
}

#[derive(FromRow, Debug)]
pub struct GroupModel {
    group_uid: GroupUID,
    weight: i32
}

impl From<GroupModel> for RustpermsOperation {
    fn from(value: GroupModel) -> Self {
        RustpermsOperation::GroupCreate{group_uid: value.group_uid, weight: value.weight}
    }
}

#[derive(FromRow, Debug)]
pub struct UserPermissionModel {
    user_uid: UserUID,
    permission: String,
    enabled: bool
}

impl FromBatch<UserPermissionModel> for RustpermsOperation {
    fn from_batch(batch: Vec<UserPermissionModel>) -> Vec<RustpermsOperation> {
        let mut m : HashMap<UserUID, Vec<PermissionRule>> = HashMap::new();
        for model in batch {
            m
                .entry(model.user_uid)
                .or_insert_with(|| Vec::with_capacity(1))
                .push((
                    PermissionPath::from_str(&model.permission),
                    model.enabled
                ));
        }
        m.into_iter().map(|(k, v)|RustpermsOperation::UserUpdatePerms(k, v)).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct GroupPermissionModel {
    group_uid: GroupUID,
    permission: String,
    enabled: bool
}

impl FromBatch<GroupPermissionModel> for RustpermsOperation {
    fn from_batch(batch: Vec<GroupPermissionModel>) -> Vec<RustpermsOperation> {
        let mut m : HashMap<GroupUID, Vec<PermissionRule>> = HashMap::new();
        for model in batch {
            m
                .entry(model.group_uid)
                .or_insert_with(|| Vec::with_capacity(1))
                .push((
                    PermissionPath::from_str(&model.permission),
                    model.enabled
                ));
        }
        m.into_iter().map(|(k, v)|RustpermsOperation::GroupUpdatePerms(k, v)).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct GroupRelationModel {
    group_uid: GroupUID,
    parent_group_uid: GroupUID
}

impl FromBatch<GroupRelationModel> for RustpermsOperation {
    fn from_batch(batch: Vec<GroupRelationModel>) -> Vec<RustpermsOperation> {
        let mut m : HashMap<GroupUID, Vec<GroupUID>> = HashMap::new();
        for model in batch {
            m
                .entry(model.group_uid)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(model.parent_group_uid);
        }
        m.into_iter().map(|(k, v)|RustpermsOperation::GroupAddParentGroups(k, v)).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct GroupUserModel {
    group_uid: GroupUID,
    user_uid: UserUID
}

impl FromBatch<GroupUserModel> for RustpermsOperation {
    fn from_batch(batch: Vec<GroupUserModel>) -> Vec<RustpermsOperation> {
        let mut m : HashMap<GroupUID, Vec<UserUID>> = HashMap::new();
        for model in batch {
            m
                .entry(model.group_uid)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(model.user_uid);
        }
        m.into_iter().map(|(k, v)|RustpermsOperation::GroupAddUsers(k, v)).collect()
    }
}



