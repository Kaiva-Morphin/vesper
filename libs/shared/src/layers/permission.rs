use std::collections::HashMap;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

use axum::extract::{MatchedPath, Path};
use axum::http::{Request, Response, StatusCode};
use axum::body::Body;
use sea_orm::ActiveValue::Set;
use tower::{Layer, Service};
use tracing::{info, warn};
use sea_orm::{sqlx, ActiveModelTrait, DbErr, EntityTrait, RuntimeErr};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;

use crate::tokens::jwt::{AccessTokenPayload, TokenEncoder};



pub enum PermissionPattern {
    NoPat{perm_id: i32},
    Pattern{pattern: String},
}


/// Permission access layer
/// You also can use patterns:
/// ```
/// route("/user/{id}", <handler>)
///     .layer(PermissionPattern::create_and_register("vesper.perm.{id}".to_string(), &db))
/// 
/// // binds {id} to {id}
/// ```
#[derive(Clone)]
pub struct PermissionAccessLayer { permission: String, id: i32 }

impl PermissionAccessLayer {
    pub async fn create_and_register(permission: String, db: &sea_orm::DatabaseConnection) -> anyhow::Result<Self> {
        permission.contains("*").then(|| panic!("Can't use wildcard in permission definition"));
        let perm = postgre_entities::permission::ActiveModel {
            name: Set(permission.clone()),
            ..Default::default()
        };

        let id = match perm.insert(db).await {
            Ok(_m) => {
                info!("Perm created!");
                _m.id
            }
            Err(DbErr::Query(RuntimeErr::SqlxError(sqlx::error::Error::Database(err)))) 
            if err.message().contains("duplicate key value") && err.message().contains("name") => {
                let p = postgre_entities::permission::Entity::find()
                .filter(postgre_entities::permission::Column::Name.eq(&permission))
                .one(db).await?;
                p.unwrap_or_else(||unreachable!()).id
            }
            Err(err) => {
                panic!("Can't create/check perm! {:#?}", err);
            }
        };
        Ok(Self {
            permission,
            id
        })
    }
}


impl<S> Layer<S> for PermissionAccessLayer {
    type Service = PermissionAccessService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        PermissionAccessService {
            service: inner,
            id: self.id,
            permission: self.permission.clone()
        }
    }
}

pub struct PermissionAccessService<S> {
    service: S,
    permission: String,
    id: i32
}

impl<S> Clone for PermissionAccessService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            id: self.id.clone(),
            permission: self.permission.clone()
        }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for PermissionAccessService<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let Some(token) = req.extensions().get::<AccessTokenPayload>() else {
            return Box::pin(async move {Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::from("Unauthorized")).unwrap())})
        };

        warn!("{:#?}", req.extensions().get::<Path<Vec<(String, String)>>>());
        let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
            let mut path_params = HashMap::new();
            for segment in matched_path.as_str().split('/') {
                if let Some((key, value)) = segment.split_once(":") {
                    path_params.insert(key.to_string(), value.to_string());
                }
            }
            info!("Matched: {:#?}", path_params);

            // MatchedPath;
            // let mut params = HashMap::new();
            // if let Some(captures) = axum::extract::Path::<(String,)>::extract(req).await.ok() {
            //     params.insert("id".to_string(), captures.0);
            // }
            // path.as_str()
        } else {
            info!("Regular: {:#?}", req.uri().path());
            // req.uri().path()
        };


        let fut = self.service.call(req);
        Box::pin(async move {
            fut.await
        })

        
        // if let Some(decoded_token) = token {
        //     let fut = self.service.call(req);
        //     Box::pin(async move {
        //         fut.await
        //     })
        // } else {
        //     Box::pin(async move {Ok(Response::builder()
        //             .status(StatusCode::UNAUTHORIZED)
        //             .body(Body::from("Unauthorized"))
        //             .unwrap())})
        // }
    }
}



