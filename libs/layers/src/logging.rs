use std::fmt::Display;

use axum::{
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use shared::utils::{hash::hash_fingerprint, header::{get_user_agent, get_user_fingerprint, get_user_ip}};
use tracing::{info, Instrument, Span};

#[derive(Clone)]
pub struct UserInfoExt {
    pub ip: String,
    pub fingerprint: String,
    pub user_agent: String
}

impl UserInfoExt {
    fn from_headers(headers: &HeaderMap) -> Self {
        Self {
            ip: get_user_ip(headers),
            fingerprint: get_user_fingerprint(headers),
            user_agent: get_user_agent(headers),
        }
    }
}

impl Display for UserInfoExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; {}; {}", self.ip, self.user_agent, hash_fingerprint(&self.fingerprint))
    }
}


pub async fn logging_middleware(mut req: Request<Body>, next: Next) -> Response {
    let span = Span::current();
    let user_info = UserInfoExt::from_headers(req.headers());
    info!("Received request on: {}. {}", req.uri().to_string(), user_info);
    // TODO! : MOVE SOMEWHERE ELSE
    req.extensions_mut().insert(user_info);
    let response = next.run(req).instrument(span).await;
    info!("Response status: {:?}", response.status());
    response
}

#[macro_export]
macro_rules! make_unique_span {
    ($name:ident) => {
        let $name = $crate::tracing::info_span!("", %format!("\x1b[90m{}\x1b[0m", $crate::uuid::Uuid::new_v4().simple()));
    };

    ($prefix:expr, $name:ident) => {
        let id = format!("\x1b[90m{}\x1b[0m", $crate::uuid::Uuid::new_v4().simple());
        let $name = $crate::tracing::info_span!($prefix, "id" = %id);
    };
}

#[macro_export]
macro_rules! layer_with_unique_span {
    ($prefix:expr) => {
        async |req: axum::extract::Request<axum::body::Body>, next: axum::middleware::Next| -> axum::response::Response {
            $crate::make_unique_span!($prefix, span);
            let response = $crate::tracing::Instrument::instrument(next.run(req), span).await;
            response
        }
    };
    () => {
        async |req: Request<Body>, next: Next| -> Response {
            let id : uuid::Uuid = uuid::Uuid::new_v4();
            $crate::make_unique_span!(span);
            let response = next.run(req).instrument(span).await;
            response
        }
    };
}
