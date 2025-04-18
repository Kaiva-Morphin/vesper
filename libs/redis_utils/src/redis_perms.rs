use crate::{redis::{RedisConn, RedisPerms}, ENV};

use anyhow::Result;
use redis::Commands;



const PERMISSION_RELATIONSHIP_PREFIX : &'static str = "PERM_REL";
fn perm_name_to_rel_key(perm_name: &str) -> String {
    format!("{}::{}", PERMISSION_RELATIONSHIP_PREFIX, perm_name)
}
fn perm_id_to_rel_key(perm: &i32) -> String {
    format!("{}::{}", PERMISSION_RELATIONSHIP_PREFIX, perm)
}

const REGISTERED_DEFAULT_PERM_PREFIX : &'static str = "REG";
fn reg_default_perm_to_key(perm: &str) -> String {
    format!("{}::{}", REGISTERED_DEFAULT_PERM_PREFIX, perm)
}

const GUEST_PERM_PREFIX : &'static str = "GUEST";
fn guest_perm_to_key(perm: &str) -> String {
    format!("{}::{}", GUEST_PERM_PREFIX, perm)
}

impl RedisPerms {
    pub fn build() -> Self {
        let redis_client = redis::Client::open(format!("redis://{}:{}/{}", ENV.REDIS_URL, ENV.REDIS_PORT, ENV.REDIS_PERMS_DB)).expect("Can't connect to redis!");
        RedisConn{
            pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }.into()
    }


    pub fn set_rel(&self, name: &String, id: &i32) -> Result<()> {
        let mut conn = self.pool.get()?;
        let _ : () = conn.mset(&[(perm_name_to_rel_key(name), &id.to_string()), (perm_id_to_rel_key(id), name)])?;
        Ok(())
    }

    pub fn set_name_to_id(&self, name: &String, id: &i32) -> Result<()> {
        let k = perm_name_to_rel_key(name);
        let mut conn = self.pool.get()?;
        let _ : () = conn.set(k, id)?;
        Ok(())
    }

    pub fn set_id_to_name(&self, name: &String, id: &i32) -> Result<()> {
        let k = perm_id_to_rel_key(id);
        let mut conn = self.pool.get()?;
        let _ : () = conn.set(k, name)?;
        Ok(())
    }

    pub fn perm_id_by_name(&self, name: String) -> Result<Option<i32>> {
        let k = perm_name_to_rel_key(&name);
        let mut conn = self.pool.get()?;
        let id: Option<i32> = conn.get(k)?;
        Ok(id)
    }
}

/*
  .oooooo.
 d8P'  `Y8b
888           oooo d8b  .ooooo.  oooo  oooo  oo.ooooo.   .oooo.o
888           `888""8P d88' `88b `888  `888   888' `88b d88(  "8
888     ooooo  888     888   888  888   888   888   888 `"Y88b.
`88.    .88'   888     888   888  888   888   888   888 o.  )88b
 `Y8bood8P'   d888b    `Y8bod8P'  `V88V"V8P'  888bod8P' 8""888P'
                                              888
                                             o888o

*/



// grp -> Zset(perm, weight*2 + !value)
// grp -> Set(grp) relations.


const GROUP_ZSET_PREFIX : &'static str = "GRP_Z";
fn group_id_to_key(group: &i32) -> String {
    format!("{}::{}", PERMISSION_RELATIONSHIP_PREFIX, group)
}

const GROUP_GROUP_RELATIONSHIP_PREFIX : &'static str = "GRP_REL";
fn grp_grp_rel_to_key(group: &i32) -> String {
    format!("{}::{}", GROUP_GROUP_RELATIONSHIP_PREFIX, group)
}

impl RedisPerms {
    pub fn set_group(
        &self,  
        group: &i32,
        weight: &i32,
        perms: Vec<(String, bool)>
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let k = group_id_to_key(group);
        
        // conn.zadd_multiple(key, items)
        Ok(())
    }
    pub fn clear_group(
        &self,
        group: &i32,
    ) -> Result<()> {
        let mut conn = self.pool.get()?;

        Ok(())
    }
    pub fn add_perm_to_group(
        &self,
        group: &i32,
        perm: &String
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let k = group_id_to_key(group);

        Ok(())
    }
    pub fn del_perm_from_group(
        &self, 
        group: &i32,
        perm: &String
    ) -> Result<()> {
        let mut conn = self.pool.get()?;
        let k = group_id_to_key(group);

        Ok(())
    }

    pub fn get_group_perms(
        &self,
        group: &i32
    ) -> Result<Vec<String>> {
        let mut conn = self.pool.get()?;
        let k = group_id_to_key(group);

        Ok(vec![])
    }
}


