use axum::{body::Body, extract::{FromRequestParts, Path}, http::request::Parts, RequestPartsExt};
use once_cell::sync::Lazy;
use regex::Regex;
use rustperms_nodes::proto::{rustperms_replica_proto_client::RustpermsReplicaProtoClient, CheckPermReply, CheckPermRequest};
use tracing::{error, info};
use std::{collections::HashMap};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use tower::{Layer, Service};
use axum::http::{Request, Response, StatusCode};
use shared::utils::IntoKey;

use shared::{tokens::jwt::AccessTokenPayload};

// Why we can't directly get a Path<Vec<(String, String)>> in middleware?
// .layer(from_extractor::<ExtractPath>()))
// At least path_extractor -> extensions -> middleware_layer works: 

#[derive(Clone, Debug)]
pub enum PermissionKind {
    NoPat{permission: String},
    Pattern{incomplete: String, replace: Vec<(String, String)>},
}

impl PermissionKind {
    pub fn try_complete(self, kvs: Option<&ExtractedPathKV>, user_id: &str) -> Option<String> {
        match self {
            Self::NoPat { permission } => Some(permission),
            Self::Pattern { incomplete, replace } => {
                let mut permission = incomplete.clone();
                if user_id != "" {
                    permission = permission.replace("{from_access}", user_id);
                }
                info!("Extracted: {:?}", kvs);
                if let Some(ExtractedPathKV(kvs)) = kvs {
                    for (in_brackets, without_brackets) in replace {
                        if let Some(v) = kvs.get(&without_brackets) {
                            permission = permission.replace(&in_brackets, v);
                        }
                    }
                }
                if permission.is_complete() {Some(permission)} else {
                    error!("Can't complete permission! Pattern: {} ~> {}", incomplete, permission);
                    None
                }
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct PermissionMiddlewareBundle {
    pub permission: PermissionKind,
    pub rustperms_client: RustpermsReplicaProtoClient<tonic::transport::Channel>,
    pub on_fail: StatusCode,
}

pub trait CompletePerm {
    fn is_complete(&self) -> bool;
}

impl CompletePerm for String {
    fn is_complete(&self) -> bool {
        !self.contains(['{', '}'])
    }
}

impl PermissionMiddlewareBundle {
    pub async fn new(permission: String, rustperms_client: RustpermsReplicaProtoClient<tonic::transport::Channel>, on_fail: StatusCode) -> anyhow::Result<Self> {
        permission.contains("*").then(|| panic!("Can't use wildcard in permission definition"));
        let c = REGEX.captures_iter(&permission)
            .filter_map(|m| m.get(1).map(|v| (format!("{{{}}}", v.as_str()), v.as_str().to_string())))
            .collect::<Vec<(String, String)>>();
        let permission = if !c.is_empty() {
            PermissionKind::Pattern { incomplete: permission, replace: c }
        } else {
            PermissionKind::NoPat { permission }
        };
        Ok(Self{permission, on_fail, rustperms_client})
    }
}



#[derive(Clone, Debug)]
pub struct PermissionAccessLayer(PermissionMiddlewareBundle);

const REGEX : Lazy<Regex> = Lazy::new(||Regex::new(r"(?:\{)([^\{\}]+)(?:\})").expect("Can't parse permission pattern regex!"));

impl PermissionAccessLayer {
    pub async fn new(permission: String, rustperms_client: RustpermsReplicaProtoClient<tonic::transport::Channel>, on_fail: StatusCode) -> anyhow::Result<Self> {
        info!("Creating permission layer for {}", permission);
        Ok(Self(PermissionMiddlewareBundle::new(permission, rustperms_client, on_fail).await?))
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
    perm_bundle: PermissionMiddlewareBundle,
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
    <S as Service<axum::http::Request<ReqBody>>>::Error: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let user_uid = if let Some(token) = req.extensions().get::<AccessTokenPayload>() {
            token.user.into_key()
        } else {
            "".to_string()
        };

        let mut client = self.perm_bundle.rustperms_client.clone();
        let on_fail = Ok(Response::builder().status(self.perm_bundle.on_fail).body(Body::empty()).unwrap());
        let kvs = req.extensions().get::<ExtractedPathKV>();
        let permission = self.perm_bundle.permission.clone().try_complete(kvs, &user_uid);
        let Some(permission) = permission else {return Box::pin(async {on_fail})};
        let next = self.service.call(req);
        Box::pin(async move { 
            info!("Starting {} check for {}", permission, if user_uid == "" {"\"guest\""} else {&user_uid});
            let reply = client.check_perm(
                CheckPermRequest{user_uid, permission, unset_policy: false}
            ).await;
            match reply {
                Ok(response) => {
                    let CheckPermReply{result: check_result} = response.into_inner();
                    info!("Perm check result: {}!", check_result);
                    if !check_result {
                        on_fail
                    } else {
                        next.await
                    }
                }
                Err(e) => {
                    error!("Can't call check perm from middleware!: {e}");
                    on_fail
                }
            }
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

/// Permission access layer builder
/// 
/// You also can use patterns, just put your var in brackets:
/// ```ignore
/// let p = PermissionMiddlewareBuilder::new(...)
/// route("/user/{id}", <handler>)
///     .layer(p.build("vesper.perm.{id}").await?)
/// // binds {id} to {id}
/// ```
/// Also, you can get guid from access token - use reserved {from_access} in your perm
/// ```ignore
/// route("/user/edit", <handler>)
///     .layer(p.build("vesper.edit.{from_access}").await?)
/// 
/// ```
/// ## DON'T CHAIN IT LIKE THAT:
/// ```ignore
/// route("/user/{id}", get(<handler>).layer(perm).post(<handler>).layer(perm2))
/// ```
pub struct PermissionMiddlewareBuilder {
    // permission: String, rustperms_client: RustpermsReplicaProtoClient<tonic::transport::Channel>, on_fail: StatusCode
    pub rustperms_client: RustpermsReplicaProtoClient<tonic::transport::Channel>,
}

impl PermissionMiddlewareBuilder {
    pub fn new(rustperms_client: RustpermsReplicaProtoClient<tonic::transport::Channel>) -> Self {
        Self {rustperms_client}
    }

    pub async fn build(&self, path: &str) -> anyhow::Result<PermissionAccessLayer> {
        PermissionAccessLayer::new(path.to_string(), self.rustperms_client.clone(), StatusCode::UNAUTHORIZED).await
    }
}