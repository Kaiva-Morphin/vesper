use rustperms::prelude::*;
use shared::env_config;

use crate::db::{PostgreStorage, ReflectedApply, SqlStore};
mod db;

env_config!(
    ".env" => ENV = Env {
        DATABASE_URL : String
    }
);


pub async fn run_rustperms_test(manager: &AsyncManager, storage: &PostgreStorage, actions: Vec<RustpermsOperation>) -> anyhow::Result<()> {
    let mut delta = RustpermsDelta::new();
    
    for op in actions {
        // use fictive users and groups for checking applying without actually applying to local, because reflected_apply requires unapplied changes and apply it both locally and in db.
        let changed = rustperms::prelude::AsyncManager::apply_action(&mut manager.users.read().await.clone(), &mut manager.groups.read().await.clone(), op.clone());
        assert!(changed, "Operation should have effect locally: {op:?}");

        let r = manager.reflected_apply(storage, RustpermsDelta::from(vec![op.clone()])).await;
        assert!(r.is_ok(), "Reflected apply failed: {op:?}");

        delta.push(op);
    }

    let reloaded = storage.load_manager().await?;
    assert!(manager.eq(&reloaded).await, "Reloaded state does not match");

    Ok(())
}

#[tokio::test]
async fn test_user_create_and_assign() -> anyhow::Result<()> {
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.drop_tables().await.ok();
    storage.init_schema().await?;

    let actions = vec![
        RustpermsOperation::UserCreate("user1".into()),
        RustpermsOperation::UserUpdatePerms("user1".into(), vec![
            (PermissionPath::from_str("test.permission"), true),
        ]),
        RustpermsOperation::GroupCreate { group_uid: "admin".into(), weight: 100 },
        RustpermsOperation::GroupAddUsers("admin".into(), vec!["user1".into()]),
    ];
    let manager = AsyncManager::default();

    run_rustperms_test(&manager, &storage, actions).await?;

    assert!(manager.users.read().await.get("user1").is_some());
    assert!(manager.users.read().await.get("user1").unwrap().get_perms().get(&PermissionPath::from_str("test.permission")).is_some());
    assert!(manager.groups.read().await.get("admin").is_some());
    assert!(manager.users.read().await.get("user1").unwrap().get_groups().get("admin").is_some());
    assert!(manager.groups.read().await.get("admin").unwrap().get_members().contains("user1"));

    Ok(())
}

#[tokio::test]
async fn test_user_create_remove() -> anyhow::Result<()> {
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.drop_tables().await.ok();
    storage.init_schema().await?;

    let actions = vec![
        RustpermsOperation::UserCreate("user1".into()),
        RustpermsOperation::UserRemove("user1".into()),
    ];
    let manager = AsyncManager::default();
    run_rustperms_test(&manager, &storage, actions).await?;
    assert!(manager.users.read().await.get("user1").is_none());

    Ok(())
}

#[tokio::test]
async fn test_group_create_and_weight_update() -> anyhow::Result<()> {
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.drop_tables().await.ok();
    storage.init_schema().await?;

    let actions = vec![
        RustpermsOperation::GroupCreate { group_uid: "mod".into(), weight: 10 },
        RustpermsOperation::GroupUpdate { group_uid: "mod".into(), weight: 20 },
    ];
    let manager = AsyncManager::default();
    run_rustperms_test(&manager, &storage, actions).await?;

    let weight = manager.groups.read().await.get("mod").unwrap().get_weight();
    assert_eq!(weight, 20);

    Ok(())
}

#[tokio::test]
async fn test_group_permission_modification() -> anyhow::Result<()> {
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.drop_tables().await.ok();
    storage.init_schema().await?;

    let path = PermissionPath::from_str("group.perm.test");

    let actions = vec![
        RustpermsOperation::GroupCreate { group_uid: "dev".into(), weight: 5 },
        RustpermsOperation::GroupUpdatePerms("dev".into(), vec![(path.clone(), true)]),
        RustpermsOperation::GroupRemovePerms("dev".into(), vec![path.clone()]),
    ];
    let manager = AsyncManager::default();
    run_rustperms_test(&manager, &storage, actions).await?;

    assert!(manager.groups.read().await.get("dev").unwrap().get_perms().get(&path).is_none());

    Ok(())
}

#[tokio::test]
async fn test_group_hierarchy() -> anyhow::Result<()> {
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.drop_tables().await.ok();
    storage.init_schema().await?;

    let actions = vec![
        RustpermsOperation::GroupCreate { group_uid: "base".into(), weight: 0 },
        RustpermsOperation::GroupCreate { group_uid: "child".into(), weight: 5 },
        RustpermsOperation::GroupAddParentGroups("child".into(), vec!["base".into()]),
        RustpermsOperation::GroupRemoveParentGroups("child".into(), vec!["base".into()]),
    ];
    let manager = AsyncManager::default();
    run_rustperms_test(&manager, &storage, actions).await?;

    assert!(!manager.groups.read().await.get("child").unwrap().get_parents().contains("base"));

    Ok(())
}



#[tokio::test]
async fn test_remove_user_from_group() -> anyhow::Result<()> {
    let storage = db::PostgreStorage::connect(&ENV.DATABASE_URL).await?;
    storage.drop_tables().await.ok();
    storage.init_schema().await?;

    let actions = vec![
        RustpermsOperation::UserCreate("alice".into()),
        RustpermsOperation::GroupCreate { group_uid: "admin".into(), weight: 100 },
        RustpermsOperation::GroupAddUsers("admin".into(), vec!["alice".into()]),
        RustpermsOperation::GroupRemoveUsers("admin".into(), vec!["alice".into()]),
    ];
    let manager = AsyncManager::default();
    run_rustperms_test(&manager, &storage, actions).await?;
    assert!(!manager.users.read().await.get("alice").unwrap().has_group(&"admin".into()));
    assert!(!manager.groups.read().await.get("admin").unwrap().has_member(&"alice".into()));
    Ok(())
}


fn main(){}