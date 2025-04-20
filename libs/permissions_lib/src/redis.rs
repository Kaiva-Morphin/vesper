use std::collections::HashMap;

use bb8_redis::redis::{AsyncCommands, RedisError};
use redis_utils::{redis::RedisConn, redis_wrapper};
use anyhow::Result;

use crate::CFG;



redis_wrapper!(RedisPerms);


impl RedisPerms {
    pub async fn default() -> Self {
        RedisConn::perms().await.into()
    }
}

const PERM_ID_REL_PREFIX : &'static str = "PERM_ID";
const ID_PERM_REL_PREFIX : &'static str = "ID_PERM";
fn perm_to_key(perm: &String) -> String {
    format!("{}::{}", PERM_ID_REL_PREFIX, perm)
    
}

fn perm_id_to_key(id: &u64) -> String {
    format!("{}::{}", ID_PERM_REL_PREFIX, id)
}

const WILDCARD_ID_REL_PREFIX : &'static str = "WILDCARD_ID";
const ID_WILDCARD_REL_PREFIX : &'static str = "ID_WILDCARD";
fn wildcard_to_key(perm: &String) -> String {
    format!("{}::{}", WILDCARD_ID_REL_PREFIX, perm)
    
}

fn wildcard_id_to_key(id: &u64) -> String {
    format!("{}::{}", ID_WILDCARD_REL_PREFIX, id)
}

// // i think i will regret this
// macro_rules! prefixed_redis{
//     ($prefix:ident) => {
//         paste::paste!{
//             impl RedisPerms {
//                 async fn update_perm_rel_lifetime(
//                     &self,
//                     c: &mut bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>,
//                     perm: &String
//                 ) -> Result<()>{
//                     let id : Option<u64> = c.get(perm_to_key(perm)).await?;
//                     if let Some(id) = id {
//                         let _ : () = c.expire(perm_id_to_key(&id), CFG.REDIS_PERM_CACHE_LIFETIME).await?;
//                     }
//                     let _ : () = c.expire(perm_to_key(perm), CFG.REDIS_PERM_CACHE_LIFETIME).await?;
//                     Ok(())
//                 }

//                 async fn update_perm_rel_lifetime_by_id(
//                     &self,
//                     c: &mut bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>,
//                     id: &u64
//                 ) -> Result<()>{
//                     let perm : Option<String> = c.get(perm_id_to_key(id)).await?;
//                     if let Some(perm) = perm {
//                         let _ : () = c.expire(perm_to_key(&perm), CFG.REDIS_PERM_CACHE_LIFETIME).await?;
//                     }
//                     let _ : () = c.expire(perm_id_to_key(id), CFG.REDIS_PERM_CACHE_LIFETIME).await?;
//                     Ok(())
//                 }

//                 pub async fn insert_perm_rel(
//                     &self,
//                     perm: &String,
//                     id: &u64,
//                 ) -> Result<()> {
//                     let mut c = self.conn.pool.get().await?;
//                     let _: () = c.mset(&[(perm_to_key(perm), &id.to_string()), (perm_id_to_key(id), perm)]).await?;
//                     self.update_perm_rel_lifetime(&mut c, perm).await?;
//                     Ok(())
//                 }

//                 pub async fn rm_perm(
//                     &self,
//                     perm: &String
//                 ) ->  Result<()> {
//                     let mut c = self.conn.pool.get().await?;
//                     let id : Option<u64> = c.get_del(perm_to_key(perm)).await?;
//                     if let Some(id) = id {
//                         let _ : () = c.del(perm_id_to_key(&id)).await?;
//                     }
//                     Ok(())
//                 }

//                 pub async fn rm_perm_by_id(
//                     &self,
//                     id: &u64
//                 ) ->  Result<()> {
//                     let mut c = self.conn.pool.get().await?;
//                     let perm : Option<String> = c.get_del(perm_id_to_key(id)).await?;
//                     if let Some(perm) = perm {
//                         let _ : () = c.del(perm_to_key(&perm)).await?;
//                     }
//                     Ok(())
//                 }

//                 pub async fn get_perm_id(&self, perm: &String) -> Result<Option<u64>> {
//                     let mut c = self.conn.pool.get().await?;
//                     let id : Option<u64> = c.get(perm_to_key(perm)).await?;
//                     self.update_perm_rel_lifetime(&mut c, perm).await?;
//                     Ok(id)
//                 }

//                 pub async fn get_perm_by_id(&self, id: &u64) -> Result<Option<String>> {
//                     let mut c = self.conn.pool.get().await?;
//                     let perm : Option<String> = c.get(perm_id_to_key(id)).await?;
//                     self.update_perm_rel_lifetime_by_id(&mut c, id).await?;
//                     Ok(perm)
//                 }

//                 pub async fn get_multiple_perm_ids(&self, perms: Vec<String>) -> Result<HashMap<String, u64>> {
//                     let mut c = self.conn.pool.get().await?;
//                     let v : Vec<Option<u64>> = c.mget(perms.iter().map(|k| perm_to_key(&k)).collect::<Vec<String>>()).await?;
//                     let mut out_map = HashMap::new();
//                     for (i, id) in v.iter().enumerate() {
//                         if let Some(id) = id {
//                             self.update_perm_rel_lifetime_by_id(&mut c, id).await?;
//                             out_map.insert(perms.get(i).unwrap().to_string(), *id);
//                         }
//                     }
//                     Ok(out_map)
//                 }

//                 pub async fn get_multiple_perms_by_ids(&self, ids: Vec<u64>) -> Result<HashMap<u64, String>> {
//                     let mut c = self.conn.pool.get().await?;
//                     let v : Vec<Option<String>> = c.mget(ids.iter().map(|k| perm_id_to_key(&k)).collect::<Vec<String>>()).await?;
//                     let mut out_map = HashMap::new();
//                     for (i, perm) in v.into_iter().enumerate() {
//                         if let Some(perm) = perm {
//                             self.update_perm_rel_lifetime(&mut c, &perm).await?;
//                             out_map.insert(*ids.get(i).unwrap(), perm);
//                         }
//                     }
//                     Ok(out_map)
//                 }

//                 pub async fn rm_multiple_perms(&self, perms: Vec<String>) -> Result<()> {
//                     if perms.is_empty() {return Ok(());}
//                     let mut c = self.conn.pool.get().await?;
//                     let perm_keys = perms.iter().map(|k| perm_to_key(&k)).collect::<Vec<String>>();
//                     let id_keys : Vec<Option<u64>> = c.mget(&perm_keys).await?;
//                     let _ : () = c.del(&perm_keys).await?;
//                     let to_delete = id_keys.into_iter().filter_map(|x| x.and_then(|v| Some(perm_id_to_key(&v)))).collect::<Vec<String>>();
//                     if !to_delete.is_empty() {
//                         let _: () = c.del(to_delete).await?;
//                     }
//                     Ok(())
//                 }

//                 pub async fn rm_multiple_perms_by_ids(&self, ids: Vec<u64>) -> Result<()> {
//                     if ids.is_empty() {return Ok(());}
//                     let mut c = self.conn.pool.get().await?;
//                     let id_keys = ids.iter().map(|k| perm_id_to_key(&k)).collect::<Vec<String>>();
//                     let perm_keys : Vec<Option<String>> = c.mget(&id_keys).await?;
//                     let _ : () = c.del(&id_keys).await?;
//                     let to_delete = perm_keys.into_iter().filter_map(|x| x.and_then(|v| Some(perm_to_key(&v)))).collect::<Vec<String>>();
//                     if !to_delete.is_empty() {
//                         let _: () = c.del(to_delete).await?;
//                     }
//                     Ok(())
//                 }
//             }
//         }
//     }
// }

// i will regret this
macro_rules! prefixed_redis{
    ($prefix:ident, $upper_prefix:ident) => {
        paste::paste!{
            impl RedisPerms {
                async fn [<update_ $prefix _rel_lifetime>](
                    &self,
                    c: &mut bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>,
                    $prefix : &String
                ) -> Result<()>{
                    let id : Option<u64> = c.get([<$prefix _to_key>]($prefix)).await?;
                    if let Some(id) = id {
                        let _ : () = c.expire([<$prefix _id_to_key>](&id), CFG.[<REDIS_ $upper_prefix _CACHE_LIFETIME>]).await?;
                    }
                    let _ : () = c.expire([<$prefix _to_key>]($prefix), CFG.[<REDIS_ $upper_prefix _CACHE_LIFETIME>]).await?;
                    Ok(())
                }

                async fn [<update_ $prefix _rel_lifetime_by_id>](
                    &self,
                    c: &mut bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>,
                    id: &u64
                ) -> Result<()>{
                    let $prefix : Option<String> = c.get([<$prefix _id_to_key>](id)).await?;
                    if let Some($prefix) =  $prefix  {
                        let _ : () = c.expire([<$prefix _to_key>](& $prefix), CFG.[<REDIS_ $upper_prefix _CACHE_LIFETIME>]).await?;
                    }
                    let _ : () = c.expire([<$prefix _id_to_key>](id), CFG.[<REDIS_ $upper_prefix _CACHE_LIFETIME>]).await?;
                    Ok(())
                }

                pub async fn [<insert_ $prefix _rel>](
                    &self,
                     $prefix : &String,
                    id: &u64,
                ) -> Result<()> {
                    let mut c = self.conn.pool.get().await?;
                    let _: () = c.mset(&[([<$prefix _to_key>]($prefix), &id.to_string()), ([<$prefix _id_to_key>](id), $prefix)]).await?;
                    self.[<update_ $prefix _rel_lifetime>](&mut c, $prefix).await?;
                    Ok(())
                }

                pub async fn [<rm_ $prefix>](
                    &self,
                     $prefix : &String
                ) ->  Result<()> {
                    let mut c = self.conn.pool.get().await?;
                    let id : Option<u64> = c.get_del([<$prefix _to_key>]($prefix)).await?;
                    if let Some(id) = id {
                        let _ : () = c.del([<$prefix _id_to_key>](&id)).await?;
                    }
                    Ok(())
                }

                pub async fn [<rm_ $prefix _by_id>](
                    &self,
                    id: &u64
                ) ->  Result<()> {
                    let mut c = self.conn.pool.get().await?;
                    let $prefix : Option<String> = c.get_del([<$prefix _id_to_key>](id)).await?;
                    if let Some($prefix) =  $prefix  {
                        let _ : () = c.del([<$prefix _to_key>](&$prefix)).await?;
                    }
                    Ok(())
                }

                pub async fn [<get_ $prefix _id>](&self, $prefix: &String) -> Result<Option<u64>> {
                    let mut c = self.conn.pool.get().await?;
                    let id : Option<u64> = c.get([<$prefix _to_key>]($prefix)).await?;
                    self.[<update_ $prefix _rel_lifetime>](&mut c, $prefix).await?;
                    Ok(id)
                }

                pub async fn [<get_ $prefix _by_id>](&self, id: &u64) -> Result<Option<String>> {
                    let mut c = self.conn.pool.get().await?;
                    let $prefix : Option<String> = c.get([<$prefix _id_to_key>](id)).await?;
                    self.[<update_ $prefix _rel_lifetime_by_id>](&mut c, id).await?;
                    Ok($prefix)
                }

                pub async fn [<get_multiple_ $prefix _ids>](&self, [<$prefix s>]: Vec<String>) -> Result<HashMap<String, u64>> {
                    let mut c = self.conn.pool.get().await?;
                    let v : Vec<Option<u64>> = c.mget([<$prefix s>].iter().map(|k| [<$prefix _to_key>](&k)).collect::<Vec<String>>()).await?;
                    let mut out_map = HashMap::new();
                    for (i, id) in v.iter().enumerate() {
                        if let Some(id) = id {
                            self.[<update_ $prefix _rel_lifetime_by_id>](&mut c, id).await?;
                            out_map.insert([<$prefix s>].get(i).unwrap().to_string(), *id);
                        }
                    }
                    Ok(out_map)
                }

                pub async fn [<get_multiple_ $prefix s_by_ids>](&self, ids: Vec<u64>) -> Result<HashMap<u64, String>> {
                    let mut c = self.conn.pool.get().await?;
                    let v : Vec<Option<String>> = c.mget(ids.iter().map(|k|[<$prefix _id_to_key>](&k)).collect::<Vec<String>>()).await?;
                    let mut out_map = HashMap::new();
                    for (i, $prefix) in v.into_iter().enumerate() {
                        if let Some($prefix) =  $prefix  {
                            self.[<update_ $prefix _rel_lifetime>](&mut c, &$prefix).await?;
                            out_map.insert(*ids.get(i).unwrap(), $prefix);
                        }
                    }
                    Ok(out_map)
                }

                pub async fn [<rm_multiple_ $prefix s>](&self, [<$prefix s>]: Vec<String>) -> Result<()> {
                    if [<$prefix s>].is_empty() {return Ok(());}
                    let mut c = self.conn.pool.get().await?;
                    let [<$prefix _keys>] = [<$prefix s>].iter().map(|k|[<$prefix _to_key>](&k)).collect::<Vec<String>>();
                    let id_keys : Vec<Option<u64>> = c.mget(&[<$prefix _keys>]).await?;
                    let _ : () = c.del(&[<$prefix _keys>]).await?;
                    let to_delete = id_keys.into_iter().filter_map(|x| x.and_then(|v| Some([<$prefix _id_to_key>](&v)))).collect::<Vec<String>>();
                    if !to_delete.is_empty() {
                        let _: () = c.del(to_delete).await?;
                    }
                    Ok(())
                }

                pub async fn [<rm_multiple_ $prefix s_by_ids>](&self, ids: Vec<u64>) -> Result<()> {
                    if ids.is_empty() {return Ok(());}
                    let mut c = self.conn.pool.get().await?;
                    let id_keys = ids.iter().map(|k|[<$prefix _id_to_key>](&k)).collect::<Vec<String>>();
                    let [<$prefix _keys>] : Vec<Option<String>> = c.mget(&id_keys).await?;
                    let _ : () = c.del(&id_keys).await?;
                    let to_delete = [<$prefix _keys>].into_iter().filter_map(|x| x.and_then(|v| Some([<$prefix _to_key>](&v)))).collect::<Vec<String>>();
                    if !to_delete.is_empty() {
                        let _: () = c.del(to_delete).await?;
                    }
                    Ok(())
                }
            }
        }
    }
}

prefixed_redis!(perm, PERM);
prefixed_redis!(wildcard, WILDCARD);