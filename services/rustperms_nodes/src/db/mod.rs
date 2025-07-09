pub mod models;
use std::collections::HashMap;

use rustperms::{api::actions::{RustpermsDelta, RustpermsOperation}, prelude::{AsyncManager, PermPath, PermissionPart, PermissionPath}};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow, Executor, IntoArguments, PgExecutor, Pool, Postgres, Transaction, Type};
use anyhow::Result;
use tokio::sync::RwLock;
use tonic::async_trait;
use tracing::error;

pub const USER_SCHEMA : &'static str = include_str!("./schema/user.sql");
pub const GROUP_SCHEMA : &'static str = include_str!("./schema/group.sql");
pub const DROP_SCHEMA : &'static str = include_str!("./schema/drop.sql");

pub struct PostgreStorage {
    conn: Pool<Postgres>
}

impl PostgreStorage {
    pub async fn connect(database: &str) -> Result<Self> {
        let conn: Pool<Postgres> = PgPoolOptions::new()
            .max_connections(8)
            .connect(database).await?;
        Ok(Self{conn})
    }

    pub async fn drop(self){
        self.conn.close().await;
    }
}

pub trait SqlQuery<DB : sqlx::Database> {
    async fn sql_query<'e>(self, e: (impl Executor<'_, Database = DB> + Send)) -> Result<()>
    where
        std::string::String: sqlx::Encode<'e, DB> + Type<DB>,
        i32: sqlx::Encode<'e, DB> + Type<DB>,
        Vec<bool>: sqlx::Encode<'e, DB> + Type<DB>,
        Vec<String>: sqlx::Encode<'e, DB> + Type<DB>,
        <DB as sqlx::Database>::Arguments<'e>: IntoArguments<'e, DB>;
}

impl<DB : sqlx::Database> SqlQuery<DB> for RustpermsOperation {
    async fn sql_query<'e>(self, e: impl Executor<'_, Database = DB> + Send) -> Result<()> 
    where
        std::string::String: sqlx::Encode<'e, DB> + Type<DB>,
        i32: sqlx::Encode<'e, DB> + Type<DB>,
        Vec<bool>: sqlx::Encode<'e, DB> + Type<DB>,
        Vec<String>: sqlx::Encode<'e, DB> + Type<DB>,
        <DB as sqlx::Database>::Arguments<'e>: IntoArguments<'e, DB>
    {
        match self {
            RustpermsOperation::UserCreate(u) => {
                tracing::info!("Creating user: {}", u);
                sqlx::query("INSERT INTO rustperms_user (user_uid) VALUES ($1) ON CONFLICT (user_uid) DO nothing")
                    .bind(u)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::UserRemove(u) => {
                sqlx::query("DELETE FROM rustperms_user where user_uid = $1")
                    .bind(u)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::UserUpdatePerms(u, ps) => {
                let mut enabled: Vec<bool> = Vec::with_capacity(ps.len());
                let mut perms: Vec<String> = Vec::with_capacity(ps.len());
                for (p, e) in ps.into_iter() {
                    perms.push(p.format());
                    enabled.push(e);
                }
                sqlx::query(r#"
                    INSERT INTO rustperms_user_permissions (user_uid, permission, enabled)
                    SELECT $1, perms.permission, perms.enabled
                    FROM UNNEST($2::text[], $3::bool[]) AS perms(permission, enabled)
                    ON CONFLICT (user_uid, permission)
                    DO UPDATE SET enabled = EXCLUDED.enabled
                "#)
                    .bind(u)
                    .bind(perms)
                    .bind(enabled)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::UserRemovePerms(u, ps) => {
                sqlx::query(r#"
                    DELETE FROM rustperms_user_permissions 
                    USING UNNEST($2::text[]) as rules(permission)
                    WHERE user_uid = $1 AND rustperms_user_permissions.permission = rules.permission
                "#)
                    .bind(u)
                    .bind(ps.into_iter().map(|p| p.format()).collect::<Vec<String>>())
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupCreate{group_uid: g, weight: w} => {
                tracing::info!("Creating group: {}", g);
                sqlx::query("INSERT INTO rustperms_group (group_uid, weight) VALUES ($1, $2) ON CONFLICT (group_uid) DO update set weight = EXCLUDED.weight")
                    .bind(g)
                    .bind(w)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupUpdate { group_uid: g, weight: w } => {
                sqlx::query("UPDATE rustperms_group set weight = $2 WHERE group_uid = $1")
                    .bind(g)
                    .bind(w)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupRemove(g) => {
                sqlx::query("DELETE FROM rustperms_group WHERE group_uid = $1")
                    .bind(g)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupUpdatePerms(g, ps) => {
                let mut enabled: Vec<bool> = Vec::with_capacity(ps.len());
                let mut perms: Vec<String> = Vec::with_capacity(ps.len());
                for (p, e) in ps.into_iter() {
                    perms.push(p.format());
                    enabled.push(e);
                }
                sqlx::query(r#"
                    INSERT INTO rustperms_group_permissions (group_uid, permission, enabled)
                    SELECT $1, perms.permission, perms.enabled
                    FROM UNNEST($2::text[], $3::bool[]) AS perms(permission, enabled)
                    ON CONFLICT (group_uid, permission)
                    DO UPDATE SET enabled = EXCLUDED.enabled
                "#)
                    .bind(g)
                    .bind(perms)
                    .bind(enabled)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupRemovePerms(g, ps) => {
                sqlx::query(r#"
                    DELETE FROM rustperms_group_permissions
                    USING UNNEST($2::text[]) as rules(permission)
                    WHERE group_uid = $1 AND rustperms_group_permissions.permission = rules.permission
                "#)
                    .bind(g)
                    .bind(ps.into_iter().map(|p| p.format()).collect::<Vec<String>>())
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupAddParentGroups(g, gs) => {
                sqlx::query(r#"
                    INSERT INTO rustperms_group_relations (group_uid, parent_group_uid)
                    SELECT $1, groups.group FROM
                    UNNEST ($2::text[]) as groups("group")
                    ON CONFLICT (group_uid, parent_group_uid) DO nothing"#)
                    .bind(g)
                    .bind(gs)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupRemoveParentGroups(g, gs) => {
                sqlx::query(r#"
                    DELETE FROM rustperms_group_relations
                    USING UNNEST($2::text[]) as groups("group")
                    WHERE group_uid = $1 AND parent_group_uid = groups.group
                "#)
                    .bind(g)
                    .bind(gs)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupAddUsers(g, us) => {
                tracing::info!("Adding {:?} to {}", us, g);
                sqlx::query(r#"
                    INSERT INTO rustperms_user_groups (group_uid, user_uid)
                    SELECT $1, users.user FROM
                    UNNEST ($2::text[]) as users("user")
                    ON CONFLICT (group_uid, user_uid) DO nothing"#)
                    .bind(g)
                    .bind(us)
                    .execute(e).await?;
                Ok(())
            }
            RustpermsOperation::GroupRemoveUsers(g, us) => {
                sqlx::query(r#"
                    DELETE FROM rustperms_user_groups
                    USING UNNEST($2::text[]) as users("user")
                    WHERE group_uid = $1 AND user_uid = users.user
                "#)
                    .bind(g)
                    .bind(us)
                    .execute(e).await?;
                Ok(())
            }
        }
    }
}


#[async_trait]
pub trait SqlStore<DB: sqlx::Database> : Sync {
    async fn begin_tx(&self) -> Result<Transaction<'_, DB>>;
    async fn init_schema(&self) -> anyhow::Result<()>;
    async fn drop_tables(&self) -> anyhow::Result<()>;
    async fn load_manager(&self) -> Result<AsyncManager>;
}

#[async_trait]
impl SqlStore<Postgres> for PostgreStorage {
    async fn begin_tx(&self) -> Result<Transaction<'_, Postgres>> {
        let tx: Transaction<'_, Postgres> = self.conn.begin().await?;
        Ok(tx)
    }
    async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::raw_sql(USER_SCHEMA).execute(&self.conn).await?;
        sqlx::raw_sql(GROUP_SCHEMA).execute(&self.conn).await?;
        Ok(())
    }
    async fn drop_tables(&self) -> anyhow::Result<()> {
        sqlx::raw_sql(DROP_SCHEMA).execute(&self.conn).await?;
        Ok(())
    }
    async fn load_manager(&self) -> Result<AsyncManager> {
        use models::*;

        let mut dt = RustpermsDelta::new();
        
        // load all users
        let s : Vec<UserModel> = sqlx::query_as("select * from rustperms_user").fetch_all(&self.conn).await?;
        dt.push_many(RustpermsOperation::from_batch(s));
        
        // load user <-> perm relations
        let s : Vec<UserPermissionModel> = sqlx::query_as("select * from rustperms_user_permissions").fetch_all(&self.conn).await?;
        dt.push_many(RustpermsOperation::from_batch(s));

        // load all groups
        let s : Vec<GroupModel> = sqlx::query_as("select * from rustperms_group").fetch_all(&self.conn).await?;
        dt.push_many(RustpermsOperation::from_batch(s));

        // load group <-> perm relations
        let s : Vec<GroupPermissionModel> = sqlx::query_as("select * from rustperms_group_permissions").fetch_all(&self.conn).await?;
        dt.push_many(RustpermsOperation::from_batch(s));

        // load group <-> group relations
        let s : Vec<GroupRelationModel> = sqlx::query_as("select * from rustperms_group_relations").fetch_all(&self.conn).await?;
        dt.push_many(RustpermsOperation::from_batch(s));

        // load user <-> group relations
        let s : Vec<GroupUserModel> = sqlx::query_as("select * from rustperms_user_groups").fetch_all(&self.conn).await?;
        dt.push_many(RustpermsOperation::from_batch(s));

        Ok(dt.into())
    }
}

#[tonic::async_trait]
pub trait ReflectedApply<DB : sqlx::Database>{
    async fn reflected_apply(&self, storage: &impl SqlStore<DB>, actions: RustpermsDelta) -> Result<()>;
}

#[tonic::async_trait]
impl ReflectedApply<Postgres> for AsyncManager {
    async fn reflected_apply(&self, storage: &impl SqlStore<Postgres>, actions: RustpermsDelta) -> Result<()>
    where 
        std::string::String: sqlx::Encode<'static, Postgres> + Type<Postgres>,
        i32: sqlx::Encode<'static, Postgres> + Type<Postgres>,
        Vec<bool>: sqlx::Encode<'static, Postgres> + Type<Postgres>,
        Vec<String>: sqlx::Encode<'static, Postgres> + Type<Postgres>,
        <Postgres as sqlx::Database>::Arguments<'static>: IntoArguments<'static, Postgres>,
        RustpermsOperation : SqlQuery<Postgres> {
        let mut users = self.users.write().await;
        let mut groups = self.groups.write().await;
        let mut tx = storage.begin_tx()
            .await
            .inspect_err(|e| error!("Can't begin transaction: {:?}", e))?; // todo: delay writes.
        
        for action in actions.into_iter() {
            if Self::apply_action(&mut users, &mut groups, action.clone()) {
                action.sql_query(&mut *tx).await
                    .inspect_err(|e| error!("Can't execute sql query for action: {:?}", e))
                    .ok();
            }
        }
        tx.commit().await
            .inspect_err(|e| error!("Can't commit changes to db: {:?}", e))
            .ok(); // todo: delay writes if errors.
        Ok(())
    }
}
