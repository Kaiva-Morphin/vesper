use anyhow::Result;
use bb8_redis::redis::AsyncCommands;
use tracing::info;
use uuid::Uuid;

use crate::redis::RedisConn;

pub const USER_GUIDS_HASH: &str = "USER_GUIDS";
pub const USER_UIDS_HASH: &str = "USER_UIDS";

pub trait RedisUsers {
    async fn get_user_guid(&self, user_uid: &str) -> Result<Option<Uuid>>;
    async fn get_user_uid(&self, user_guid: &Uuid) -> Result<Option<String>>;
    async fn fill_users(&self, users: Vec<(Uuid, String)>) -> Result<()>;
    async fn add_user(&self, user_guid: &Uuid, user_uid: &str) -> Result<()>;
    async fn remove_user(&self, user_guid: &Uuid, user_uid: &str) -> Result<()>;
    async fn get_user_guids(&self) -> Result<Vec<Uuid>>;
}

impl RedisUsers for RedisConn {
    async fn fill_users(&self, users: Vec<(Uuid, String)>) -> Result<()> {
        let mut conn = self.pool.get().await?;

        let mut guid_to_uid: Vec<(String, String)> = Vec::new();
        let mut uid_to_guid: Vec<(String, String)> = Vec::new();

        for (guid, uid) in users {
            info!("Adding user {} {}", guid, uid);
            guid_to_uid.push((guid.simple().to_string(), uid.clone()));
            uid_to_guid.push((uid, guid.simple().to_string()));
        }

        let _: () = conn.hset_multiple(USER_GUIDS_HASH, &guid_to_uid).await?;
        let _: () = conn.hset_multiple(USER_UIDS_HASH, &uid_to_guid).await?;
        Ok(())
    }

    async fn get_user_guids(&self) -> Result<Vec<Uuid>> {
        let mut conn = self.pool.get().await?;
        let keys: Vec<String> = conn.hkeys(USER_GUIDS_HASH).await?;
        Ok(keys.into_iter().map(|k| k.parse().unwrap()).collect())
    }

    async fn get_user_guid(&self, user_uid: &str) -> Result<Option<Uuid>> {
        let mut conn = self.pool.get().await?;
        let guid_str: Option<String> = conn.hget(USER_UIDS_HASH, user_uid).await?;
        Ok(guid_str.map(|g| g.parse()).transpose()?)
    }

    async fn get_user_uid(&self, user_guid: &Uuid) -> Result<Option<String>> {
        let mut conn = self.pool.get().await?;
        let uid: Option<String> = conn.hget(USER_GUIDS_HASH, user_guid.simple().to_string()).await?;
        Ok(uid)
    }

    async fn add_user(&self, user_guid: &Uuid, user_uid: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        info!("Adding user {} {}", user_guid, user_uid);
        let _: () = conn.hset(USER_GUIDS_HASH, user_guid.simple().to_string(), user_uid).await?;
        let _: () = conn.hset(USER_UIDS_HASH, user_uid, user_guid.simple().to_string()).await?;
        Ok(())
    }

    async fn remove_user(&self, user_guid: &Uuid, user_uid: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        info!("Removing user {} {}", user_guid, user_uid);
        let _: () = conn.hdel(USER_GUIDS_HASH, user_guid.simple().to_string()).await?;
        let _: () = conn.hdel(USER_UIDS_HASH, user_uid).await?;
        Ok(())
    }
}
