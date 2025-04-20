use core::error;
use std::collections::{HashMap, HashSet};

use sea_orm::ActiveValue::Set;
use sea_orm::{sqlx, ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter, RuntimeErr, TryInsertResult};
use shared::env_config;
use anyhow::Result;
use tracing::{info, warn, error};

pub mod middleware;
pub mod redis;

#[cfg(test)]
pub mod tests;




env_config!(
    ".cfg" => CFG = EnvCfg{
        REDIS_PERM_CACHE_LIFETIME : i64 = 60 * 10, // 10 min
        REDIS_WILDCARD_CACHE_LIFETIME : i64 = 60 * 20, // 20 min
        REDIS_CONTAINER_CACHE_LIFETIME : i64 = 60 * 60, // 1h
    }
);



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

    pub async fn insert_perm(&self, perm: &String) -> Result<()> {
        if self.redis.get_perm_id(perm).await?.is_some() { return Ok(()); }
        let p = postgre_entities::permission::ActiveModel {
            path: Set(perm.to_string()),
            ..Default::default()
        };
        let _r = postgre_entities::permission::Entity::insert(p).on_conflict_do_nothing().exec(&self.db).await?;
        Ok(())        
    }

    async fn get_perm_from_db(&self, perm: &String) -> Result<Option<postgre_entities::permission::Model>> {
        let p = postgre_entities::permission::Entity::find().filter(postgre_entities::permission::Column::Path.eq(perm)).one(&self.db).await?;
        Ok(p)
    }

    async fn get_perm_from_db_by_id(&self, id: u64) -> Result<Option<postgre_entities::permission::Model>> {
        let p = postgre_entities::permission::Entity::find().filter(postgre_entities::permission::Column::PermId.eq(id)).one(&self.db).await?;
        Ok(p)
    }

    pub async fn get_perm_id(&self, perm: &String) -> Result<Option<u64>> {
        let id = self.redis.get_perm_id(perm).await?;
        if let Some(id) = id { return Ok(Some(id));}
        let p = self.get_perm_from_db(perm).await?;
        let Some(m) = p else {return Ok(None)};
        let id = m.perm_id as u64;
        self.redis.insert_perm_rel(perm, &id).await?;
        Ok(Some(id))
    }

    pub async fn get_perm_by_id(&self, id: &u64) -> Result<Option<String>> {
        let perm = self.redis.get_perm_by_id(id).await?;
        if let Some(perm) = perm { return Ok(Some(perm));}
        let p = self.get_perm_from_db_by_id(*id).await?;
        let Some(m) = p else {return Ok(None)};
        let perm = m.path;
        self.redis.insert_perm_rel(&perm, id).await?;
        Ok(Some(perm))
    }

    pub async fn remove_perm(&self, perm: &String) -> Result<()> {
        let p = self.get_perm_from_db(perm).await?;
        let Some(m) = p else {return Ok(())};
        m.into_active_model().delete(&self.db).await?;
        self.redis.rm_perm(perm).await?;
        Ok(())
    }
    pub async fn remove_perm_by_id(&self, id: &u64) -> Result<()> {
        let p = self.get_perm_from_db_by_id(*id).await?;
        let Some(m) = p else {return Ok(())};
        m.into_active_model().delete(&self.db).await?;
        self.redis.rm_perm_by_id(id).await?;
        Ok(())
    }

    pub async fn get_multiple_perms_ids(&self, perms: Vec<String>) -> Result<HashMap<String, u64>> {
        let mut cached = self.redis.get_multiple_perm_ids(perms.clone()).await?;
        for perm in perms {
            if cached.contains_key(&perm) {continue;}
            let id = self.get_perm_id(&perm).await?; // This will update the cache, get_perm_from_db will not
            if let Some(id) = id {
                cached.insert(perm, id);
            }
        }
        Ok(cached)
    }

    pub async fn get_multiple_perms_by_ids(&self, ids: Vec<u64>) -> Result<HashMap<u64, String>> {
        let mut cached = self.redis.get_multiple_perms_by_ids(ids.clone()).await?;
        for id in ids {
            if cached.contains_key(&id) {continue;}
            let perm = self.get_perm_by_id(&id).await?;
            if let Some(perm) = perm {
                cached.insert(id, perm);
            }
        }
        Ok(cached)
    }

    pub async fn insert_multiple_perms(&self, perms: Vec<String>) -> Result<()> {
        let mut entities = vec![];
        let mut pushed = HashSet::new();
        for perm in perms {
            if (self.redis.get_perm_id(&perm).await?).is_some() {continue;}
            if pushed.contains(&perm) {continue;};
            pushed.insert(perm.clone());
            entities.push(
                postgre_entities::permission::ActiveModel {
                    path: Set(perm.to_string()),
                    ..Default::default()
                }
            );
        }
        postgre_entities::permission::Entity::insert_many(entities)
            .on_conflict_do_nothing()
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn remove_multiple_perms(&self, perms: Vec<String>) -> Result<()> {
        postgre_entities::permission::Entity::delete_many()
            .filter(postgre_entities::permission::Column::Path.is_in(&perms))
            .exec(&self.db)
            .await?;
        self.redis.rm_multiple_perms(perms).await?;
        Ok(())
    }

    pub async fn remove_multiple_perms_by_id(&self, ids: Vec<u64>) -> Result<()> {
        postgre_entities::permission::Entity::delete_many()
            .filter(postgre_entities::permission::Column::PermId.is_in(ids.iter().cloned().map(|v| v as i64).collect::<Vec<i64>>()))
            .exec(&self.db)
            .await?;
        self.redis.rm_multiple_perms_by_ids(ids).await?;
        Ok(())
    }


}

/*
oooooo   oooooo     oooo  o8o  oooo        .o8                                     .o8
 `888.    `888.     .8'   `"'  `888       "888                                    "888
  `888.   .8888.   .8'   oooo   888   .oooo888   .ooooo.   .oooo.   oooo d8b  .oooo888   .oooo.o
   `888  .8'`888. .8'    `888   888  d88' `888  d88' `"Y8 `P  )88b  `888""8P d88' `888  d88(  "8
    `888.8'  `888.8'      888   888  888   888  888        .oP"888   888     888   888  `"Y88b.
     `888'    `888'       888   888  888   888  888   .o8 d8(  888   888     888   888  o.  )88b
      `8'      `8'       o888o o888o `Y8bod88P" `Y8bod8P' `Y888""8o d888b    `Y8bod88P" 8""888P'



*/
impl Permissions {
    pub async fn insert_wildcard(&self, wildcard: &String) -> Result<()> {
        if self.redis.get_wildcard_id(wildcard).await?.is_some() { return Ok(()); }
        let p = postgre_entities::perm_wildcard::ActiveModel {
            path: Set(wildcard.to_string()),
            ..Default::default()
        };
        let _r = postgre_entities::perm_wildcard::Entity::insert(p).on_conflict_do_nothing().exec(&self.db).await?;
        Ok(())        
    }

    async fn get_wildcard_from_db(&self, wildcard: &String) -> Result<Option<postgre_entities::perm_wildcard::Model>> {
        let p = postgre_entities::perm_wildcard::Entity::find().filter(postgre_entities::perm_wildcard::Column::Path.eq(wildcard)).one(&self.db).await?;
        Ok(p)
    }

    async fn get_wildcard_from_db_by_id(&self, id: u64) -> Result<Option<postgre_entities::perm_wildcard::Model>> {
        let p = postgre_entities::perm_wildcard::Entity::find().filter(postgre_entities::perm_wildcard::Column::PermWildcardId.eq(id)).one(&self.db).await?;
        Ok(p)
    }

    pub async fn get_wildcard_id(&self, wildcard: &String) -> Result<Option<u64>> {
        let id = self.redis.get_wildcard_id(wildcard).await?;
        if let Some(id) = id { return Ok(Some(id));}
        let p = self.get_wildcard_from_db(wildcard).await?;
        let Some(m) = p else {return Ok(None)};
        let id = m.perm_wildcard_id as u64;
        self.redis.insert_wildcard_rel(wildcard, &id).await?;
        Ok(Some(id))
    }

    pub async fn get_wildcard_by_id(&self, id: &u64) -> Result<Option<String>> {
        let wildcard = self.redis.get_wildcard_by_id(id).await?;
        if let Some(wildcard) = wildcard { return Ok(Some(wildcard));}
        let p = self.get_wildcard_from_db_by_id(*id).await?;
        let Some(m) = p else {return Ok(None)};
        let wildcard = m.path;
        self.redis.insert_wildcard_rel(&wildcard, id).await?;
        Ok(Some(wildcard))
    }

    pub async fn remove_wildcard(&self, wildcard: &String) -> Result<()> {
        let p = self.get_wildcard_from_db(wildcard).await?;
        let Some(m) = p else {return Ok(())};
        m.into_active_model().delete(&self.db).await?;
        self.redis.rm_wildcard(wildcard).await?;
        Ok(())
    }
    pub async fn remove_wildcard_by_id(&self, id: &u64) -> Result<()> {
        let p = self.get_wildcard_from_db_by_id(*id).await?;
        let Some(m) = p else {return Ok(())};
        m.into_active_model().delete(&self.db).await?;
        self.redis.rm_wildcard_by_id(id).await?;
        Ok(())
    }

    pub async fn get_multiple_wildcards_ids(&self, wildcards: Vec<String>) -> Result<HashMap<String, u64>> {
        let mut cached = self.redis.get_multiple_wildcard_ids(wildcards.clone()).await?;
        for wildcard in wildcards {
            if cached.contains_key(&wildcard) {continue;}
            let id = self.get_wildcard_id(&wildcard).await?; // This will update the cache, get_wildcard_from_db will not
            if let Some(id) = id {
                cached.insert(wildcard, id);
            }
        }
        Ok(cached)
    }

    pub async fn get_multiple_wildcards_by_ids(&self, ids: Vec<u64>) -> Result<HashMap<u64, String>> {
        let mut cached = self.redis.get_multiple_wildcards_by_ids(ids.clone()).await?;
        for id in ids {
            if cached.contains_key(&id) {continue;}
            let wildcard = self.get_wildcard_by_id(&id).await?;
            if let Some(wildcard) = wildcard {
                cached.insert(id, wildcard);
            }
        }
        Ok(cached)
    }

    pub async fn insert_multiple_wildcards(&self, wildcards: Vec<String>) -> Result<()> {
        let mut entities = vec![];
        let mut pushed = HashSet::new();
        for wildcard in wildcards {
            if (self.redis.get_wildcard_id(&wildcard).await?).is_some() {continue;}
            if pushed.contains(&wildcard) {continue;};
            pushed.insert(wildcard.clone());
            entities.push(
                postgre_entities::perm_wildcard::ActiveModel {
                    path: Set(wildcard.to_string()),
                    ..Default::default()
                }
            );
        }
        postgre_entities::perm_wildcard::Entity::insert_many(entities)
            .on_conflict_do_nothing()
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn remove_multiple_wildcards(&self, wildcards: Vec<String>) -> Result<()> {
        postgre_entities::perm_wildcard::Entity::delete_many()
            .filter(postgre_entities::perm_wildcard::Column::Path.is_in(&wildcards))
            .exec(&self.db)
            .await?;
        self.redis.rm_multiple_wildcards(wildcards).await?;
        Ok(())
    }

    pub async fn remove_multiple_wildcards_by_id(&self, ids: Vec<u64>) -> Result<()> {
        postgre_entities::perm_wildcard::Entity::delete_many()
            .filter(postgre_entities::perm_wildcard::Column::PermWildcardId.is_in(ids.iter().cloned().map(|v| v as i64).collect::<Vec<i64>>()))
            .exec(&self.db)
            .await?;
        self.redis.rm_multiple_wildcards_by_ids(ids).await?;
        Ok(())
    }
}





