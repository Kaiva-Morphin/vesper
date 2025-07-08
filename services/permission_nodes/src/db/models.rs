use std::collections::HashMap;

use rustperms::{api::actions::PermissionOp, prelude::*};
use sqlx::prelude::*;

pub trait FromBatch<T> where Self: Sized {
    fn from_batch(batch: Vec<T>) -> Vec<Self>;
}

impl<T: Into<PermissionOp>> FromBatch<T> for PermissionOp {
    fn from_batch(batch: Vec<T>) -> Vec<PermissionOp> {
        batch.into_iter().map(|u| u.into()).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct UserModel {
    username: String
}

impl From<UserModel> for PermissionOp {
    fn from(value: UserModel) -> Self {
        PermissionOp::UserCreate(value.username)
    }
}

#[derive(FromRow, Debug)]
pub struct GroupModel {
    groupname: Groupname,
    weight: i32
}

impl From<GroupModel> for PermissionOp {
    fn from(value: GroupModel) -> Self {
        PermissionOp::GroupCreate{groupname: value.groupname, weight: value.weight}
    }
}

#[derive(FromRow, Debug)]
pub struct UserPermissionModel {
    username: Username,
    permission: String,
    enabled: bool
}

impl FromBatch<UserPermissionModel> for PermissionOp {
    fn from_batch(batch: Vec<UserPermissionModel>) -> Vec<PermissionOp> {
        let mut m : HashMap<Username, Vec<PermissionRule>> = HashMap::new();
        for model in batch {
            m
                .entry(model.username)
                .or_insert_with(|| Vec::with_capacity(1))
                .push((
                    PermissionPath::from_str(&model.permission),
                    model.enabled
                ));
        }
        m.into_iter().map(|(k, v)|PermissionOp::UserUpdatePerms(k, v)).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct GroupPermissionModel {
    groupname: Groupname,
    permission: String,
    enabled: bool
}

impl FromBatch<GroupPermissionModel> for PermissionOp {
    fn from_batch(batch: Vec<GroupPermissionModel>) -> Vec<PermissionOp> {
        let mut m : HashMap<Groupname, Vec<PermissionRule>> = HashMap::new();
        for model in batch {
            m
                .entry(model.groupname)
                .or_insert_with(|| Vec::with_capacity(1))
                .push((
                    PermissionPath::from_str(&model.permission),
                    model.enabled
                ));
        }
        m.into_iter().map(|(k, v)|PermissionOp::GroupUpdatePerms(k, v)).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct GroupRelationModel {
    groupname: Groupname,
    parent_groupname: Groupname
}

impl FromBatch<GroupRelationModel> for PermissionOp {
    fn from_batch(batch: Vec<GroupRelationModel>) -> Vec<PermissionOp> {
        let mut m : HashMap<Groupname, Vec<Groupname>> = HashMap::new();
        for model in batch {
            m
                .entry(model.groupname)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(model.parent_groupname);
        }
        m.into_iter().map(|(k, v)|PermissionOp::GroupAddParentGroups(k, v)).collect()
    }
}

#[derive(FromRow, Debug)]
pub struct GroupUserModel {
    groupname: Groupname,
    username: Username
}

impl FromBatch<GroupUserModel> for PermissionOp {
    fn from_batch(batch: Vec<GroupUserModel>) -> Vec<PermissionOp> {
        tracing::warn!("From batch called on group user model! : {:#?}", batch);
        let mut m : HashMap<Groupname, Vec<Username>> = HashMap::new();
        for model in batch {
            m
                .entry(model.groupname)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(model.username);
        }
        m.into_iter().map(|(k, v)|PermissionOp::GroupAddUsers(k, v)).collect()
    }
}



