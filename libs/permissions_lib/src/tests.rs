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

// macro_rules! make_perm_test {
//     (fn $test_name:ident(){$($body:tt)*}) => {
//         #[tokio::test]
//         async fn $test_name() {
//             let p = Permissions::test().await;
//             $($body)*
//         }
//     };
// }


// make_perm_test!{
//     fn permission_add(){
//         let perm = "vesper.test.permission.permission_add".to_string();
//         p.perm_insert(perm).await;
//         assert_eq!(true, p.exists(perm).await);
//     }
// }


macro_rules! test_perm_bundle {
    ($prefix:ident) => {
        paste::paste! {
            #[tokio::test]
            async fn [<$prefix _insert>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.add", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(id.is_some(), true);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _insert_twice>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.insert_twice", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                let id1 = p.[<get_ $prefix _id>](&perm).await?;
                p.[<insert_ $prefix>](&perm).await?;
                let id2 = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(id1.is_some(), true);
                assert_eq!(id2.is_some(), true);
                assert_eq!(id2, id1);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _insert_get>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.insert_get", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?.unwrap();
                let path = p.[<get_ $prefix _by_id>](&id).await?;
                assert_eq!(Some(perm), path);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _remove>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.remove", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                p.[<remove_ $prefix>](&perm).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(false, id.is_some());
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _remove_twice>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.remove_twice", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                p.[<remove_ $prefix>](&perm).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(false, id.is_some());
                p.[<remove_ $prefix>](&perm).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(false, id.is_some());
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _remove_get>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.remove_get", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                let id1 = p.[<get_ $prefix _id>](&perm).await?;
                p.[<remove_ $prefix>](&perm).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(false, id.is_some());
                let path = p.[<get_ $prefix _by_id>](&id1.unwrap()).await?;
                assert_eq!(None, path);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _remove_by_id_get>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.remove_by_id_get", stringify!($prefix));
                p.[<insert_ $prefix>](&perm).await?;
                let id1 = p.[<get_ $prefix _id>](&perm).await?;
                p.[<remove_ $prefix _by_id>](&id1.unwrap()).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(false, id.is_some());
                let path = p.[<get_ $prefix _by_id>](&id1.unwrap()).await?;
                assert_eq!(None, path);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_insert>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_insert1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_insert2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_insert3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_insert4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                for perm in perms.iter() {
                    let id = p.[<get_ $prefix _id>](&perm).await?;
                    assert_eq!(id.is_some(), true);
                    let path = p.[<get_ $prefix _by_id>](&id.unwrap()).await?;
                    assert_eq!(Some(perm.to_string()), path);
                }
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_insert_same>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.multiple_insert_same", stringify!($prefix));
                let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let id = p.[<get_ $prefix _id>](&perm).await?;
                assert_eq!(id.is_some(), true);
                let path = p.[<get_ $prefix _by_id>](&id.unwrap()).await?;
                assert_eq!(Some(perm.to_string()), path);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_get>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_get1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_get2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_get3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_get4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids : HashMap<String, u64> = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?;
                assert_eq!(ids.len(), perms.len());
                for perm_key in ids.keys() {
                    let id = p.[<get_ $prefix _id>](perm_key).await?;
                    assert_eq!(id.is_some(), true);
                    let path = p.[<get_ $prefix _by_id>](&id.unwrap()).await?;
                    assert_eq!(Some(perm_key.to_string()), path);
                }
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_get_same>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.multiple_get_same", stringify!($prefix));
                let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids : HashMap<String, u64> = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?;
                assert_eq!(ids.len(), 1);
                for perm_key in ids.keys() {
                    let id = p.[<get_ $prefix _id>](perm_key).await?;
                    assert_eq!(id.is_some(), true);
                    let path = p.[<get_ $prefix _by_id>](&id.unwrap()).await?;
                    assert_eq!(Some(perm_key.to_string()), path);
                }
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_get_by_id>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_get_by_id1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_get_by_id2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_get_by_id3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_get_by_id4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), perms.len());
                let id_to_perms : HashMap<u64, String> = p.[<get_multiple_ $prefix s_by_ids>](ids.clone()).await?;
                assert_eq!(id_to_perms.len(), perms.len());
                for id in id_to_perms.keys() {
                    let path = p.[<get_ $prefix _by_id>](&id).await?;
                    assert_eq!(id_to_perms.get(id).cloned(), path);
                }
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_get_by_id_same>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.multiple_get_by_id_same", stringify!($prefix));
                let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 1);
                let ids = vec![ids[0].clone(), ids[0].clone(), ids[0].clone(), ids[0].clone()];
                let id_to_perms : HashMap<u64, String> = p.[<get_multiple_ $prefix s_by_ids>](ids).await?;
                assert_eq!(id_to_perms.len(), 1);
                for id in id_to_perms.keys() {
                    let path = p.[<get_ $prefix _by_id>](&id).await?;
                    assert_eq!(id_to_perms.get(id).cloned(), path);
                }
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_remove>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_remove1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                p.[<remove_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 0);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_remove_same>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.multiple_remove_same", stringify!($prefix));
                let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                p.[<remove_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 0);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_remove_by_id>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_remove_by_id1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_by_id2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_by_id3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_by_id4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                p.[<remove_multiple_ $prefix s_by_id>](ids).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 0);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_remove_by_id_same>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perm = format!("{}.vesper.test.multiple_remove_by_id_same", stringify!($prefix));
                let perms = vec![perm.clone(), perm.clone(), perm.clone(), perm.clone()];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 1);
                let ids = vec![ids[0].clone(), ids[0].clone(), ids[0].clone(), ids[0].clone()];
                p.[<remove_multiple_ $prefix s_by_id>](ids).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 0);
                Ok(())
            }
            
            #[tokio::test]
            async fn [<$prefix _multiple_remove_twice>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_remove_twice1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_twice2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_twice3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_twice4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                p.[<remove_multiple_ $prefix s>](perms.clone()).await?;
                p.[<remove_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 0);
                Ok(())
            }

            #[tokio::test]
            async fn [<$prefix _multiple_remove_by_id_twice>]() -> Result<()> {
                let p = Permissions::for_test().await;
                let perms = vec![
                    format!("{}.vesper.test.multiple_remove_by_id_twice1", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_by_id_twice2", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_by_id_twice3", stringify!($prefix)),
                    format!("{}.vesper.test.multiple_remove_by_id_twice4", stringify!($prefix))
                ];
                p.[<insert_multiple_ $prefix s>](perms.clone()).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                p.[<remove_multiple_ $prefix s_by_id>](ids.clone()).await?;
                p.[<remove_multiple_ $prefix s_by_id>](ids).await?;
                let ids = p.[<get_multiple_ $prefix s_ids>](perms.clone()).await?.values().cloned().collect::<Vec<u64>>();
                assert_eq!(ids.len(), 0);
                Ok(())
            }
        }
    };
}

test_perm_bundle!{perm}
test_perm_bundle!{wildcard}


// #[tokio::test]
// async fn permission_add() {
//     let p = Permissions::test().await;
//     let perm = "vesper.test.permission.permission_add".to_string();
//     p.perm_insert(perm).await;
//     assert_eq!(true, p.exists(perm).await);
// }

// #[tokio::test]
// async fn permission_add_exists() {
//     let p = Permissions::test().await;
//     let perm = "vesper.test.permission.permission_add_exists".to_string();
//     p.perm_insert(perm).await;
//     p.perm_insert(perm).await;
//     assert_eq!(true, p.exists(perm).await);
// }

// #[tokio::test]
// async fn permission_remove() {
//     let p = Permissions::test().await;
//     let perm = "vesper.test.permission.permission_remove".to_string();
//     p.perm_remove(perm).await;
//     p.perm_insert(perm).await;
//     p.perm_remove(perm).await;
//     assert_eq!(false, p.perm_exists(perm).await);
// }


// #[tokio::test]
// async fn permission_insert_rm_many() {
//     let p = Permissions::test().await;
//     let perms = (0..10).map(|n| format!("vesper.test.permission.permission_insert_rm_many_{}", n)).collect::<Vec<String>>();
//     p.perm_insert_many(perms).await;
//     for perm in perms.iter() {
//         assert_eq!(true, p.perm_exists(perm.clone()).await);
//     }
//     p.perm_remove_many(perms).await;
//     for perm in perms.iter() {
//         assert_eq!(false, p.perm_exists(perm.clone()).await);
//     }
// }

// #[tokio::test]
// async fn wildcard_add() {
//     let p = Permissions::test().await;
//     let perm = "vesper.test.wildcard.wildcard_add".to_string();
//     p.wildcard_insert(perm).await;
//     assert_eq!(true, p.wildcard_exists(perm).await);
// }

// #[tokio::test]
// async fn wildcard_add_exists() {
//     let p = Permissions::test().await;
//     let perm = "vesper.test.wildcard.wildcard_add_exists".to_string();
//     p.wildcard_insert(perm).await;
//     p.wildcard_insert(perm).await;
//     assert_eq!(true, p.wildcard_exists(perm).await);
// }

// #[tokio::test]
// async fn wildcard_remove() {
//     let p = Permissions::test().await;
//     let perm = "vesper.test.wildcard.wildcard_remove".to_string();
//     p.wildcard_remove(perm).await;
//     p.wildcard_insert(perm).await;
//     p.wildcard_remove(perm).await;
//     assert_eq!(false, p.wildcard_exists(perm).await);
// }


// #[tokio::test]
// async fn wildcard_insert_rm_many() {
//     let p = Permissions::test().await;
//     let perms = (0..10).map(|n| format!("vesper.test.wildcard.wildcard_insert_rm_many_{}", n)).collect::<Vec<String>>();
//     p.wildcard_insert_many(perms).await;
//     for perm in perms.iter() {
//         assert_eq!(true, p.wildcard_exists(perm.clone()).await);
//     }
//     p.wildcard_remove_many(perms).await;
//     for perm in perms.iter() {
//         assert_eq!(false, p.wildcard_exists(perm.clone()).await);
//     }
// }
