use axum::{body::Body, extract::{FromRequestParts, Path}, http::request::Parts, RequestPartsExt};
use once_cell::sync::Lazy;
use regex::Regex;
use sqlx_adapter::casbin::{CoreApi, Enforcer, Model};
use tracing::{error, info, warn};
use std::collections::HashMap;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use tower::{Layer, Service};
use axum::http::{Request, Response, StatusCode};
use shared::{tokens::jwt::AccessTokenPayload, utils::set_encoder::decode_set_from_string};
use anyhow::Result;
use crate::PermissionDB;

pub struct PermissionAccessLayerBuilder<'a> {
    perm_db: &'a PermissionDB,
}

impl<'a> PermissionAccessLayerBuilder<'a> {
    pub fn new(perm_db: &'a PermissionDB) -> Self {
        Self {
            perm_db: perm_db
        }
    }
    pub async fn layer(self, permission: String) -> Result<PermissionAccessLayer> {
        Ok(PermissionAccessLayer::new(permission, self.perm_db.clone()).await?)
    }
}

// Why we can't directly get a Path<Vec<(String, String)>> in middleware?
// At least path_extractor -> extensions -> middleware_layer works:

pub trait PermissionChecker {
    fn check(&self, perm: &String) -> bool;
}

impl PermissionChecker for AccessTokenPayload {
    fn check(&self, perm: &String) -> bool {
        unimplemented!();
    }
}

#[derive(Clone)]
pub struct PermissionBundle {
    pub path: String,
    pub pattern: Vec<(String, String)>,
    pub perm_db: PermissionDB,
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
    pub async fn new(permission: &'static str, perm_db: PermissionDB) -> anyhow::Result<Self> {
        permission.contains("*").then(|| panic!("Can't use wildcard in permission definition"));
        let c = REGEX.captures_iter(permission)
            .filter_map(|m| m.get(1).map(|v| (format!("{{{}}}", v.as_str()), v.as_str().to_string())))
            .collect::<Vec<(String, String)>>();

        Ok(PermissionBundle{
            path: permission.to_string(),
            pattern: c,
            on_fail: StatusCode::UNAUTHORIZED,
            perm_db: perm_db.clone()
        })
    }

    // perm_id -> Set(allowed_groups | allowed_users)
    pub async fn try_pass(&self, access: Option<&AccessTokenPayload>, kvs: Option<&ExtractedPathKV>) -> Result<(), StatusCode> {
        let user_id = access.map(|p| p.user.to_string()).unwrap_or("guest".to_string());


        info!("Starting perm check for {}", user_id);
        let Some(resource) = self.try_substitute(kvs) else {
            return Err(self.on_fail)
        };
        // let a = self.perm_db.0;
        // let m = Model::
        // if access.check(&self.path) {
        //     info!("Perm check passed!");
        // }
        // let mut e = Enforcer::new(m, a).await?;

        info!("Perm check passed!");
        // info!("Perm: {} ~> {:?}", self.path, self.compute(kvs));
        
        
        //if let Some() =  self.redis.perm_id_by_name(name)


        // Ok(())
        Err(self.on_fail)
    }

    pub fn try_substitute(&self, kvs: Option<&ExtractedPathKV>) -> Option<String> {
        if self.pattern.is_empty() {return Some(self.path.clone())}
        let mut permission = self.path.clone();
        if let Some(ExtractedPathKV(kvs)) = kvs {
            for (in_brackets, without_brackets) in &self.pattern {
                if let Some(v) = kvs.get(without_brackets) {
                    permission = permission.replace(in_brackets, v);
                }
            }
        }
        if permission.is_complete() {Some(permission)} else {
            error!("Can't complete permission! Pattern: {} ~> {}", self.path, permission);
            None
        }
    }
}


/// Permission access layer.
/// 
/// You also can use patterns, just put your var in brackets:
/// ```ignore
/// route("/user/{id}", <handler>)
///     .layer(PermissionAccessLayer::new("vesper.perm.{id}".to_string(), &db))
/// 
/// // binds {id} to {id}
/// ```
/// ## DON'T CHAIN IT LIKE THAT:
/// ```ignore
/// route("/user/{id}", get(<handler>).layer(perm).post(<handler>).layer(perm2))
/// ```
#[derive(Clone)]
pub struct PermissionAccessLayer(PermissionBundle);

const REGEX : Lazy<Regex> = Lazy::new(||Regex::new(r"(?:\{)([^\{\}]+)(?:\})").expect("Can't parse permission pattern regex!"));

impl PermissionAccessLayer {
    pub async fn new(permission: String, perm_db: PermissionDB) -> anyhow::Result<Self> {
        // Ok(Self(PermissionBundle::new(permission, perm_db).await?))
        unimplemented!()
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
        let token = req.extensions().get::<AccessTokenPayload>();
        unimplemented!();
        // if let Err(sc) = self.perm_bundle.try_pass(token, req.extensions().get::<ExtractedPathKV>()) {
        //     return Box::pin(async move {Ok(Response::builder().status(sc).body(Body::from(())).unwrap())})
        // };
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