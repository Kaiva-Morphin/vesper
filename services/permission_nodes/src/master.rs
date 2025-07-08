use rustperms::{api::actions::PermissionDelta, prelude::*};
use ::shared::env_config;
use anyhow::Result;
use tracing::info;

use crate::db::{ReflectedApply, SqlStore};
mod shared;
mod db;

env_config!(
    ".env" => ENV = Env {
        NATS_URL : String,
        NATS_PORT : String,
        PERM_WRITE_NATS_EVENT : String,
        DATABASE_URL : String,
        PERM_MASTER_GRPC_PORT: u16 = 3000,
    }
);

pub struct AppState {

}


#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::layer::SubscriberExt::with(tracing_subscriber::registry(), tracing_subscriber::fmt::layer());
    tracing::subscriber::set_global_default(subscriber).ok();
    
    // connect to bd
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;

    storage.init_schema().await?;
    let manager = storage.load_manager().await?;

    // println!("Manager: {:?}", p);

    let mut actions = PermissionDelta::new();

    use rustperms::api::actions::PermissionOp::*;

    actions.push(UserCreate("test_user".to_string()));

    let perms = (0..25).into_iter().map(
        |i| (PermissionPath::from_str(&format!("test.permission.{}", i)), i % 2 == 0)
    ).collect();
    actions.push(UserUpdatePerms("test_user".to_string(), perms));

    for (i, g) in [
        "guest".to_string(),
        "user".to_string(),
        "admin".to_string(),
    ].iter().enumerate() {
        actions.push(GroupCreate{groupname: g.clone(), weight: i as i32 * 10});
        actions.push(GroupUpdatePerms(g.clone(), vec![(PermissionPath::from_str(&format!("perm.{}.test", g)), true)]));
        actions.push(GroupAddUsers(g.clone(), vec!["test_user".to_string()]));
        if i != 0 {
            actions.push(GroupAddParentGroups(g.clone(), vec!["guest".to_string()]));
        }
    }
    manager.reflected_apply(&storage, actions).await?;
    info!("Send: {:#?}", manager);
    let manager = storage.load_manager().await?;
    info!("Get : {:#?}", manager);



    

    // fetch tree state

    // start nats event listener

    // start grpc listener
    Ok(())
    
}
