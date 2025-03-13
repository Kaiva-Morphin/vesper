use axum::{body::Body, extract::{FromRequestParts, Path}, http::request::Parts, RequestPartsExt};
use once_cell::sync::Lazy;
use redis_utils::redis::RedisPerms;
use regex::Regex;
use tracing::{error, info, warn};
use std::collections::HashMap;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use tower::{Layer, Service};
use sea_orm::{sqlx, ActiveModelTrait, DbErr, EntityTrait, RuntimeErr};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use axum::http::{Request, Response, StatusCode};

use shared::{tokens::jwt::AccessTokenPayload, utils::set_encoder::decode_set_from_string};


// Why we can't directly get a Path<Vec<(String, String)>> in middleware?
// At least path_extractor -> extensions -> middleware_layer works:

#[derive(Clone)]
pub enum PermissionPattern {
    NoPat{perm_id: i32},
    Pattern{replace: Vec<(String, String)>},
}


#[derive(Clone)]
pub struct PermissionBundle {
    pub perm: String,
    pub pat: PermissionPattern,
    pub db: DatabaseConnection,
    pub redis: RedisPerms,
    pub on_fail: StatusCode
}

pub trait CompletePerm {
    fn is_complete(&self) -> bool;
}

impl CompletePerm for String {
    fn is_complete(&self) -> bool {
        !self.contains(['{', '}'])
    }
}

impl PermissionBundle {

    pub async fn new(permission: String, db: &sea_orm::DatabaseConnection, redis: &RedisPerms) -> anyhow::Result<Self> {
        permission.contains("*").then(|| panic!("Can't use wildcard in permission definition"));
        let c = REGEX.captures_iter(&permission)
        .filter_map(|m| m.get(1).map(|v| (format!("{{{}}}", v.as_str()), v.as_str().to_string())))
        .collect::<Vec<(String, String)>>();
        let pattern = 
        if c.len() == 0 {
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
                    // can it all be single-queried?
                    // this vvv doesn't give id on conflict, but looks a bit cleaner than ^^^
                    // let v = postgre_entities::permission::Entity::insert(perm).on_conflict_do_nothing().exec(db).await;
                    // upd: it works strangely - .on_conflict_do_nothing() ignores and return DbErr::Query instead of TryInsertError::Conflict (like ^^^)
                    let p = postgre_entities::permission::Entity::find()
                    .filter(postgre_entities::permission::Column::Name.eq(&permission))
                    .one(db).await?;
                    p.unwrap_or_else(||unreachable!()).id
                }
                Err(err) => {
                    panic!("Can't create/check perm! {:#?}", err);
                }
            };
            redis.set_rel(&permission, &id)?;
            PermissionPattern::NoPat { perm_id: id }
        } else {
            PermissionPattern::Pattern { replace: c }
        };
        Ok(PermissionBundle{perm: permission, pat: pattern, on_fail: StatusCode::UNAUTHORIZED, db: db.clone(), redis: redis.clone()})
    }

    // perm_id -> Set(allowed_groups | allowed_users)
    pub fn try_pass(&self, access: &AccessTokenPayload, kvs: Option<&ExtractedPathKV>) -> Result<(), StatusCode> {
        let user_id = access.user;
        info!("Starting perm check for {} with {} (encoded)", user_id, access.groups);
        let Some(groups) = decode_set_from_string(&access.groups) else {
            error!("Can't parse groups!");
            return Err(self.on_fail)
        };
        info!("Perm check passed!");
        info!("{}", self.perm);
        info!("computed: {:?}", self.compute(kvs));

        
        //if let Some() =  self.redis.perm_id_by_name(name)


        // Ok(())
        Err(self.on_fail)
    }
    // provided by endpoint
    fn defined_perm(){
        
    }

    // from path, can be none, must be registered by provider
    fn undefined_perm(){
        
    }

    pub fn compute(&self, kvs: Option<&ExtractedPathKV>) -> Option<String> {
        match &self.pat {
            PermissionPattern::Pattern { replace } => {
                let mut permission = self.perm.clone();
                if let Some(ExtractedPathKV(kvs)) = kvs {
                    for (in_brackets, without_brackets) in replace {
                        if let Some(v) = kvs.get(without_brackets) {
                            permission = permission.replace(in_brackets, v);
                        }
                    }
                }
                if permission.is_complete() {Some(permission)} else {
                    error!("Can't complete permission! Pattern: {} ~> {}", self.perm, permission);
                    None
                }
            }
            _ => Some(self.perm.clone())
        }
    }

    pub fn try_substitute(&self, kvs: Option<&ExtractedPathKV>) -> Option<String> {
        match &self.pat {
            PermissionPattern::Pattern { replace } => {
                let mut permission = self.perm.clone();
                if let Some(ExtractedPathKV(kvs)) = kvs {
                    for (in_brackets, without_brackets) in replace {
                        if let Some(v) = kvs.get(without_brackets) {
                            permission = permission.replace(in_brackets, v);
                        }
                    }
                }
                if permission.is_complete() {Some(permission)} else {
                    error!("Can't complete permission! Pattern: {} ~> {}", self.perm, permission);
                    None
                }
            }
            _ => None
        }
    }
    pub fn non_dynamic(&self) -> Option<i32> {
        match &self.pat {
            PermissionPattern::NoPat { perm_id } => Some(*perm_id),
            _ => None
        }
    }
}

/// Permission access layer.
/// 
/// You also can use patterns, just put your var in brackets:
/// ```
/// route("/user/{id}", <handler>)
///     .layer(PermissionAccessLayer::create_and_register("vesper.perm.{id}".to_string(), &db))
/// 
/// // binds {id} to {id}
/// ```
/// ## DON'T CHAIN IT LIKE THAT:
/// ```
/// route("/user/{id}", get(<handler>).layer(perm).post(<handler>).layer(perm2))
/// ```
#[derive(Clone)]
pub struct PermissionAccessLayer(PermissionBundle);

const REGEX : Lazy<Regex> = Lazy::new(||Regex::new(r"(?:\{)([^\{\}]+)(?:\})").expect("Can't parse permission pattern regex!"));

impl PermissionAccessLayer {
    pub async fn new(permission: String, db: &sea_orm::DatabaseConnection, redis: &RedisPerms) -> anyhow::Result<Self> {
        Ok(Self(PermissionBundle::new(permission, db, redis).await?))
    }

    pub fn hidden(mut self) -> Self {
        self.0.on_fail = StatusCode::NOT_FOUND;
        self
    }
}

impl<S> Layer<S> for PermissionAccessLayer {
    type Service = PermissionAccessService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        PermissionAccessService {
            service: inner,
            perm_bundle: self.0.clone()
        }
    }
}

pub struct PermissionAccessService<S> {
    service: S,
    perm_bundle: PermissionBundle,
}

impl<S> Clone for PermissionAccessService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            perm_bundle: self.perm_bundle.clone(),
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

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let Some(token) = req.extensions().get::<AccessTokenPayload>() else {
            return Box::pin(async move {Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::from("Unauthorized")).unwrap())})
        };
        if let Err(sc) = self.perm_bundle.try_pass(token, req.extensions().get::<ExtractedPathKV>()) {
            return Box::pin(async move {Ok(Response::builder().status(sc).body(Body::from(())).unwrap())})
        };
        let fut = self.service.call(req);
        Box::pin(async move {
            fut.await
        })
    }
}

#[derive(Clone, Debug)]
pub struct ExtractedPathKV(pub HashMap<String, String>);

pub struct ExtractPath;

impl<S> FromRequestParts<S> for ExtractPath
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let p = parts.extract::<Path<HashMap<String, String>>>().await;
        info!("Extractor executed!");
        if let Ok(Path(p)) = p {
            info!("Extracted: {:?}!", p);
            parts.extensions.insert(ExtractedPathKV(p));
        };
        Ok(Self)
    }
}