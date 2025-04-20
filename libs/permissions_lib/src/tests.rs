use migration::MigratorTrait;
use redis_utils::redis::RedisConn;
use sea_orm::ConnectOptions;

use super::*;

env_config!(
    ".env" => TEST_ENV = Env {
        TEST_DATABASE_URL : String,
        TEST_REDIS_DB : u8,
        TEST_REDIS_PORT : u16,
        TEST_REDIS_URL : String
    }
);

impl Permissions {
    async fn for_test() -> Self {
        let mut options = ConnectOptions::new(&TEST_ENV.TEST_DATABASE_URL);
        options.sqlx_logging(false);
        let conn = sea_orm::Database::connect(options).await.expect("Can't connect to database!");
        migration::Migrator::up(&conn, None).await.ok();
        Self {
            redis: RedisConn::new(format!("redis://{}:{}/{}", TEST_ENV.TEST_REDIS_URL, TEST_ENV.TEST_REDIS_PORT, TEST_ENV.TEST_REDIS_DB)).await.into(),
            db: conn,
        }
    }
}



// TODO: USE RTEST FOR TESTING WILDCARDS
#[tokio::test]
async fn insert() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.add", stringify!($prefix)));
    p.insert(&perm).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(id.is_some(), true);
    Ok(())
}

#[tokio::test]
async fn insert_twice() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.insert_twice", stringify!($prefix)));
    p.insert(&perm).await?;
    let id1 = p.get_id(&perm).await?;
    p.insert(&perm).await?;
    let id2 = p.get_id(&perm).await?;
    assert_eq!(id1.is_some(), true);
    assert_eq!(id2.is_some(), true);
    assert_eq!(id2, id1);
    Ok(())
}

#[tokio::test]
async fn insert_get() -> Result<()> {
    let p = Permissions::for_test().await;
    let raw_perm = format!("{}.vesper.test.insert_get", stringify!($prefix));
    let perm = Perm(raw_perm.clone());
    p.insert(&perm).await?;
    let id = p.get_id(&perm).await?.unwrap();
    let path = p.get_by_id(&PermId(id)).await?;
    assert_eq!(Some(raw_perm), path);
    Ok(())
}

#[tokio::test]
async fn remove() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.remove", stringify!($prefix)));
    p.insert(&perm).await?;
    p.remove(&perm).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(false, id.is_some());
    Ok(())
}

#[tokio::test]
async fn remove_twice() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.remove_twice", stringify!($prefix)));
    p.insert(&perm).await?;
    p.remove(&perm).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(false, id.is_some());
    p.remove(&perm).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(false, id.is_some());
    Ok(())
}

#[tokio::test]
async fn remove_get() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.remove_get", stringify!($prefix)));
    p.insert(&perm).await?;
    let id1 = p.get_id(&perm).await?;
    p.remove(&perm).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(false, id.is_some());
    let path = p.get_by_id(&PermId(id1.unwrap())).await?;
    assert_eq!(None, path);
    Ok(())
}

#[tokio::test]
async fn remove_by_id_get() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.remove_by_id_get", stringify!($prefix)));
    p.insert(&perm).await?;
    let id1 = p.get_id(&perm).await?;
    p.remove_by_id(&PermId(id1.unwrap())).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(false, id.is_some());
    let path = p.get_by_id(&PermId(id1.unwrap())).await?;
    assert_eq!(None, path);
    Ok(())
}

#[tokio::test]
async fn multiple_insert() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_insert1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_insert2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_insert3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_insert4", stringify!($prefix)))
    ];
    p.insert_many(perms.clone()).await?;
    for perm in perms.iter() {
        let id = p.get_id(perm).await?;
        assert_eq!(id.is_some(), true);
        let path = p.get_by_id(&PermId(id.unwrap())).await?;
        assert_eq!(Some(perm.value().to_string()), path);
    }
    Ok(())
}

#[tokio::test]
async fn multiple_insert_same() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.multiple_insert_same", stringify!($prefix)));
    let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
    p.insert_many(perms.clone()).await?;
    let id = p.get_id(&perm).await?;
    assert_eq!(id.is_some(), true);
    let path = p.get_by_id(&PermId(id.unwrap())).await?;
    assert_eq!(Some(perm.value().to_string()), path);
    Ok(())
}

#[tokio::test]
async fn multiple_get() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_get1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_get2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_get3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_get4", stringify!($prefix)))
    ];
    p.insert_many(perms.clone()).await?;
    let ids : HashMap<String, u64> = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), perms.len());
    for perm_key in ids.keys() {
        let id = p.get_id(&Perm(perm_key.clone())).await?;
        assert_eq!(id.is_some(), true);
        let path = p.get_by_id(&PermId(id.unwrap())).await?;
        assert_eq!(Some(perm_key.to_string()), path);
    }
    Ok(())
}

#[tokio::test]
async fn multiple_get_same() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.multiple_get_same", stringify!($prefix)));
    let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
    p.insert_many(perms.clone()).await?;
    let ids : HashMap<String, u64> = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), 1);
    for perm_key in ids.keys() {
        let id = p.get_id(&Perm(perm_key.clone())).await?;
        assert_eq!(id.is_some(), true);
        let path = p.get_by_id(&PermId(id.unwrap())).await?;
        assert_eq!(Some(perm_key.to_string()), path);
    }
    Ok(())
}

#[tokio::test]
async fn multiple_get_by_id() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_get_by_id1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_get_by_id2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_get_by_id3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_get_by_id4", stringify!($prefix)))
    ];
    p.insert_many(perms.clone()).await?;
    let ids = p.get_many_ids(&perms).await?.values().cloned().collect::<Vec<u64>>();
    assert_eq!(ids.len(), perms.len());
    let id_to_perms : HashMap<u64, String> = p.get_many_by_ids(&ids.iter().map(|v|PermId(*v)).collect()).await?;
    assert_eq!(id_to_perms.len(), perms.len());
    for id in id_to_perms.keys() {
        let path = p.get_by_id(&PermId(*id)).await?;
        assert_eq!(id_to_perms.get(id).cloned(), path);
    }
    Ok(())
}

#[tokio::test]
async fn multiple_get_by_id_same() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.multiple_get_by_id_same", stringify!($prefix)));
    let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
    p.insert_many(perms.clone()).await?;
    let ids = p.get_many_ids(&perms).await?.values().cloned().collect::<Vec<u64>>();
    assert_eq!(ids.len(), 1);
    let ids = vec![ids[0].clone(), ids[0].clone(), ids[0].clone(), ids[0].clone()];
    let id_to_perms : HashMap<u64, String> = p.get_many_by_ids(&ids.iter().map(|v|PermId(*v)).collect()).await?;
    assert_eq!(id_to_perms.len(), 1);
    for id in id_to_perms.keys() {
        let path = p.get_by_id(&PermId(*id)).await?;
        assert_eq!(id_to_perms.get(id).cloned(), path);
    }
    Ok(())
}

#[tokio::test]
async fn multiple_remove() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_remove1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove4", stringify!($prefix))),
    ];
    p.insert_many(perms.clone()).await?;
    p.remove_many(&perms).await?;
    let ids = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), 0);
    Ok(())
}

#[tokio::test]
async fn multiple_remove_same() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.multiple_remove_same", stringify!($prefix)));
    let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
    p.insert_many(perms.clone()).await?;
    p.remove_many(&perms).await?;
    let ids = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), 0);
    Ok(())
}

#[tokio::test]
async fn multiple_remove_by_id() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_remove_by_id1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_by_id2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_by_id3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_by_id4", stringify!($prefix))),
    ];
    p.insert_many(perms.clone()).await?;
    let ids = p.get_many_ids(&perms).await?.values().cloned().map(|v| PermId(v)).collect();
    p.remove_many_by_id(&ids).await?;
    let ids = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), 0);
    Ok(())
}

#[tokio::test]
async fn multiple_remove_by_id_same() -> Result<()> {
    let p = Permissions::for_test().await;
    let perm = Perm(format!("{}.vesper.test.multiple_remove_by_id_same", stringify!($prefix)));
    let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
    p.insert_many(perms.clone()).await?;
    let ids = p.get_many_ids(&perms).await?.values().cloned().map(|v| PermId(v)).collect::<Vec<PermId>>();
    assert_eq!(ids.len(), 1);
    let ids = vec![ids[0].clone(), ids[0].clone(), ids[0].clone(), ids[0].clone()];
    p.remove_many_by_id(&ids).await?;
    let ids = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), 0);
    Ok(())
}

#[tokio::test]
async fn multiple_remove_twice() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_remove_twice1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_twice2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_twice3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_twice4", stringify!($prefix)))
    ];
    p.insert_many(perms.clone()).await?;
    p.remove_many(&perms).await?;
    p.remove_many(&perms).await?;
    let ids = p.get_many_ids(&perms).await?;
    assert_eq!(ids.len(), 0);
    Ok(())
}

#[tokio::test]
async fn multiple_remove_by_id_twice() -> Result<()> {
    let p = Permissions::for_test().await;
    let perms = vec![
        Perm(format!("{}.vesper.test.multiple_remove_by_id_twice1", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_by_id_twice2", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_by_id_twice3", stringify!($prefix))),
        Perm(format!("{}.vesper.test.multiple_remove_by_id_twice4", stringify!($prefix)))
    ];
    p.insert_many(perms.clone()).await?;
    let ids = p.get_many_ids(&perms).await?.values().cloned().map(|v| PermId(v)).collect::<Vec<PermId>>();
    p.remove_many_by_id(&ids).await?;
    p.remove_many_by_id(&ids).await?;
    let ids = p.get_many_ids(&perms).await?.values().cloned().collect::<Vec<u64>>();
    assert_eq!(ids.len(), 0);
    Ok(())
}