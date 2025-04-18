use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

use axum::http::{Request, Response, StatusCode};
use axum::body::Body;
use tower::{Layer, Service};

use shared::tokens::jwt::{AccessTokenPayload, TokenEncoder};
use tracing::info;

#[derive(Clone)]
pub struct AuthAccessLayer {}

impl<S> Layer<S> for AuthAccessLayer {
    type Service = AuthAccessService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        AuthAccessService {
            service: inner
        }
    }
}

pub struct AuthAccessService<S> {
    service: S,
}

impl<S> Clone for AuthAccessService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
}


impl<S, ReqBody> Service<Request<ReqBody>> for AuthAccessService<S>
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
        info!("Auth checks...");
        let auth_header = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|token| token.to_string());
        let token : Option<AccessTokenPayload> = if let Some(header_value) = auth_header {
            // if let Some(token) = header_value.strip_prefix("Bearer ") {
                TokenEncoder::decode_access(header_value.to_string())
            // } else {None}
        } else {None};
        if let Some(decoded_token) = token {
            info!("Auth passed. User: {}", decoded_token.user);
            req.extensions_mut().insert(decoded_token);
            let fut = self.service.call(req);
            Box::pin(async move {
                fut.await
            })
        } else {
            info!("Auth failed!");
            Box::pin(async move {Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from("Unauthorized"))
                    .unwrap())})
        }
    }
}


