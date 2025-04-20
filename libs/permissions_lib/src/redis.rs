use std::collections::HashMap;

use bb8_redis::redis::{AsyncCommands, RedisError};
use redis_utils::{redis::RedisConn, redis_wrapper};
use anyhow::{Context, Result};

use crate::CFG;
use crate::structs::*;



redis_wrapper!(RedisPerms);


impl RedisPerms {
    pub async fn default() -> Self {
        RedisConn::perms().await.into()
    }
}



impl Lifetime for Wildcard {fn get_lifetime(&self) -> i64 {CFG.REDIS_WILDCARD_CACHE_LIFETIME}}
impl Lifetime for WildcardId {fn get_lifetime(&self) -> i64 {CFG.REDIS_WILDCARD_CACHE_LIFETIME}}
impl Lifetime for Perm {fn get_lifetime(&self) -> i64 {CFG.REDIS_PERM_CACHE_LIFETIME}}
impl Lifetime for PermId {fn get_lifetime(&self) -> i64 {CFG.REDIS_PERM_CACHE_LIFETIME}}



impl RedisPerms {
    async fn update_rel_lifetime(
        &self,
        c: &mut bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>,
        path: &(impl Path + Lifetime),
    ) -> Result<()>{
        let k = path.to_key();
        if let Some(id) = c.get(&k).await? {
            let id = path.construct_id(id);
            let _ : () = c.expire(id.to_key(), id.get_lifetime()).await?;
        }
        let _ : () = c.expire(k, path.get_lifetime()).await?;
        Ok(())
    }

    async fn update_rel_lifetime_by_id(
        &self,
        c: &mut bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>,
        id: &(impl Id + Lifetime),
    ) -> Result<()>{
        let k = id.to_key();
        if let Some(path) = c.get(&k).await? {
            let path = id.construct_path(path);
            let _ : () = c.expire(path.to_key(), path.get_lifetime()).await?;
        }
        let _ : () = c.expire(k, id.get_lifetime()).await?;
        Ok(())
    }

    pub async fn insert_rel (
        &self,
        path : &(impl Path + Lifetime),
        id: &(impl Id + Lifetime),
    ) -> Result<()> {
        let mut c = self.conn.pool.get().await?;
        let _: () = c.mset(&[(path.to_key(), id.value().to_string().as_str()), (id.to_key(), path.value())]).await?;
        self.update_rel_lifetime(&mut c, path).await?;
        Ok(())
    }

    pub async fn remove_rel(
        &self,
        path: &(impl Path + Lifetime),
    ) ->  Result<()> {
        let mut c = self.conn.pool.get().await?;
        let id : Option<u64> = c.get_del(path.to_key()).await?;
        if let Some(id) = id {
            let _ : () = c.del(path.construct_id(id).to_key()).await?;
        }
        Ok(())
    }


    pub async fn remove_rel_by_id(
        &self,
        id: &(impl Id + Lifetime),
    ) ->  Result<()> {
        let mut c = self.conn.pool.get().await?;
        let v : Option<String> = c.get_del(id.to_key()).await?;
        if let Some(v) =  v  {
            let _ : () = c.del(id.construct_path(v).to_key()).await?;
        }
        Ok(())
    }

    pub async fn get_id(
        &self,
        path: &(impl Path + Lifetime),
    ) -> Result<Option<u64>> {
        let mut c = self.conn.pool.get().await?;
        let id : Option<u64> = c.get(path.to_key()).await?;
        self.update_rel_lifetime(&mut c, path).await?;
        Ok(id)
    }

    pub async fn get_by_id(
        &self,
        id: &(impl Id + Lifetime)
    ) -> Result<Option<String>> {
        let mut c = self.conn.pool.get().await?;
        let v : Option<String> = c.get(id.to_key()).await?;
        self.update_rel_lifetime_by_id(&mut c, id).await?;
        Ok(v)
    }

    pub async fn get_many_ids(
        &self,
        paths: &Vec<impl Path + Lifetime>
    ) -> Result<HashMap<String, u64>> {
        if paths.is_empty() {return Ok(HashMap::new());}
        let mut c = self.conn.pool.get().await?;
        let v : Vec<Option<u64>> = c.mget(paths.iter().map(|p|p.to_key()).collect::<Vec<String>>()).await?;
        let mut out_map = HashMap::new();
        for (i, id) in v.iter().enumerate() {
            if let Some(id) = id {
                let path = paths.get(i).unwrap();
                let id = path.construct_id(*id);
                self.update_rel_lifetime_by_id(&mut c, &id).await?;
                out_map.insert(path.value().to_string(), id.value());
            }
        }
        Ok(out_map)
    }

    pub async fn get_many_by_ids(
        &self,
        ids: &Vec<impl Id + Lifetime>
    ) -> Result<HashMap<u64, String>> {
        let mut c = self.conn.pool.get().await?;
        let v : Vec<Option<String>> = c.mget(ids.iter().map(|i|i.to_key()).collect::<Vec<String>>()).await?;
        let mut out_map = HashMap::new();
        for (i, path) in v.into_iter().enumerate() {
            if let Some(path) = path {
                let id = ids.get(i).unwrap();
                let path = id.construct_path(path);
                self.update_rel_lifetime(&mut c, &path).await?;
                out_map.insert(id.value(), path.value().to_string());
            }
        }
        Ok(out_map)
    }

    pub async fn remove_many(
        &self,
        paths: &Vec<impl Path + Lifetime>
    ) -> Result<()> {
        if paths.is_empty() {return Ok(());}
        let mut c = self.conn.pool.get().await?;
        let path = paths.get(0).unwrap();
        let paths_keys = paths.iter().map(|p|p.to_key()).collect::<Vec<String>>();
        let ids : Vec<Option<u64>> = c.mget(&paths_keys).await?;
        let _ : () = c.del(&paths_keys).await?;
        let to_delete = ids.into_iter().filter_map(|x| x.and_then(|v| Some(path.construct_id(v).to_key()))).collect::<Vec<String>>();
        if !to_delete.is_empty() {
            let _: () = c.del(to_delete).await?;
        }
        Ok(())
    }

    pub async fn remove_many_by_ids(
        &self,
        ids: &Vec<impl Id + Lifetime>
    ) -> Result<()> {
        if ids.is_empty() {return Ok(());}
        let mut c = self.conn.pool.get().await?;
        let id = ids.get(0).unwrap();
        let id_keys = ids.iter().map(|i|i.to_key()).collect::<Vec<String>>();
        let paths : Vec<Option<String>> = c.mget(&id_keys).await?;
        let _ : () = c.del(&id_keys).await?;
        let to_delete = paths.into_iter().filter_map(|x| x.and_then(|p| Some(id.construct_path(p).to_key()))).collect::<Vec<String>>();
        if !to_delete.is_empty() {
            let _: () = c.del(to_delete).await?;
        }
        Ok(())
    }
}