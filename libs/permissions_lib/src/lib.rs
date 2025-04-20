use core::error;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use sea_orm::ActiveValue::Set;
use sea_orm::{sqlx, ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, QueryFilter, RuntimeErr, TryInsertResult};
use shared::env_config;
use anyhow::Result;
use tracing::{info, warn, error};

pub mod middleware;

pub mod redis;
pub mod structs;
use structs::*;

#[cfg(test)]
pub mod tests;




env_config!(
    ".cfg" => CFG = EnvCfg{
        REDIS_PERM_CACHE_LIFETIME : i64 = 60 * 10, // 10 min
        REDIS_WILDCARD_CACHE_LIFETIME : i64 = 60 * 20, // 20 min
        REDIS_CONTAINER_CACHE_LIFETIME : i64 = 60 * 60, // 1h
    }
);

#[derive(Clone)]
pub struct Permissions {
    redis: redis::RedisPerms,
    db: sea_orm::DatabaseConnection
}

impl Permissions {
    pub async fn new(conn: sea_orm::DatabaseConnection, redis: redis::RedisPerms) -> Self {
        Self {
            redis: redis,
            db: conn,
        }
    }

    pub async fn insert(&self, path: &(impl Path + Lifetime + DbInsert)) -> Result<()> {
        if self.redis.get_id(path).await?.is_some() { return Ok(()); }
        path.insert(&self.db).await?;
        Ok(())        
    }

    pub async fn get_id(&self, path: &(impl Path + Lifetime + DbGet)) -> Result<Option<u64>> {
        if let Some(id) = self.redis.get_id(path).await? { return Ok(Some(id));}
        let id = path.get_id_from_db(&self.db).await?;
        let Some(id) = id else {return Ok(None)};
        self.redis.insert_rel(path, &id).await?;
        Ok(Some(id.value()))
    }

    pub async fn get_by_id(&self, id: &(impl Id + Lifetime + DbGet)) -> Result<Option<String>> {
        if let Some(perm) = self.redis.get_by_id(id).await? { return Ok(Some(perm));}
        let path =  id.get_path_from_db(&self.db).await?;
        let Some(path) = path else {return Ok(None)};
        self.redis.insert_rel(&path, id).await?;
        Ok(Some(path.value().to_string()))
    }

    pub async fn remove(&self, path: &(impl Path + Lifetime + DbDelete)) -> Result<()> {
        path.delete(&self.db).await?;
        self.redis.remove_rel(path).await?;
        Ok(())
    }
    pub async fn remove_by_id(&self, id: &(impl Id + Lifetime + DbDelete)) -> Result<()> {
        id.delete(&self.db).await?;
        self.redis.remove_rel_by_id(id).await?;
        Ok(())
    }

    pub async fn get_many_ids(&self, perms: &Vec<impl Path + Lifetime + DbGet>) -> Result<HashMap<String, u64>> {
        let mut cached = self.redis.get_many_ids(&perms).await?;
        for perm in perms {
            let path = perm.value();
            if cached.contains_key(path) {continue;}
            let id = perm.get_id_from_db(&self.db).await?;
            if let Some(id) = id {
                self.redis.insert_rel(perm, &id).await?;
                cached.insert(path.to_string(), id.value());
            }
        }
        Ok(cached)
    }

    pub async fn get_many_by_ids(&self, ids: &Vec<impl Id + Lifetime + DbGet>) -> Result<HashMap<u64, String>> {
        let mut cached = self.redis.get_many_by_ids(&ids).await?;
        for id in ids {
            let id_v = id.value();
            if cached.contains_key(&id_v) {continue;}
            let path = id.get_path_from_db(&self.db).await?;
            if let Some(path) = path {
                self.redis.insert_rel(&path, id).await?;
                cached.insert(id_v, path.value().to_string());
            }
        }
        Ok(cached)
    }

    pub async fn insert_many(&self, paths: Vec<impl Path + Lifetime + DbInsert       + Debug + Clone>) -> Result<()> {
        let cached_paths = self.redis.get_many_ids(&paths).await?;
        let iterator = paths.clone().into_iter().filter(|v|{!cached_paths.contains_key(v.value())}).collect();
        DbInsert::insert_many(&self.db, iterator).await?;
        Ok(())
    }

    pub async fn remove_many(&self, paths: &Vec<impl Path + Lifetime + DbDelete + Clone>) -> Result<()> {
        DbDelete::delete_many(&self.db, paths.clone()).await?; // i think we need to rm it from db first, idk how to do it without cloning
        self.redis.remove_many(&paths).await?;
        Ok(())
    }


    pub async fn remove_many_by_id(&self, ids: &Vec<impl Id + Lifetime + DbDelete + Clone>) -> Result<()> {
        DbDelete::delete_many(&self.db, ids.clone()).await?;
        self.redis.remove_many_by_ids(&ids).await?;
        Ok(())
    }
}