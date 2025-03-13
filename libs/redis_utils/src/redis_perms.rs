use crate::{redis::{RedisConn, RedisPerms}, ENV};

use anyhow::Result;
use redis::Commands;




// PERM::<i32> -> (ZSet(Allowed users/groups), ZSet(Denied users/groups))
const PERMISSION_SET_PREFIX : &'static str = "PERM_GRP";
fn perm_to_group_key(perm: i32) -> String {
    format!("{}::{}", PERMISSION_SET_PREFIX, perm)
}


const PERMISSION_RELATIONSHIP_PREFIX : &'static str = "PERM_REL";
fn perm_name_to_rel_key(perm_name: &String) -> String {
    format!("{}::{}", PERMISSION_RELATIONSHIP_PREFIX, perm_name)
}
fn perm_id_to_rel_key(perm: &i32) -> String {
    format!("{}::{}", PERMISSION_RELATIONSHIP_PREFIX, perm)
}

impl RedisPerms {
    pub fn for_perms() -> Self {
        let redis_client = redis::Client::open(format!("redis://{}:{}/{}", ENV.REDIS_URL, ENV.REDIS_PORT, ENV.REDIS_PERMS_DB)).expect("Can't connect to redis!");
        RedisConn{
            pool: r2d2::Pool::builder().build(redis_client).expect("Can't create pool for redis!")
        }.into()
    }

    pub fn set_rel(&self, name: &String, id: &i32) -> Result<()> {
        let mut conn = self.pool.get()?;
        let _ : () = conn.set(perm_name_to_rel_key(name), id)?;
        let _ : () = conn.set(perm_id_to_rel_key(id), name)?;
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


        Ok(None)
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



impl RedisPerms {
    pub fn set_group(){}
    pub fn add_perm_to_group(){}
    pub fn del_perm_from_group(){}
}