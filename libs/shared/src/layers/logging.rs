use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use tracing::{info, info_span, Instrument, Span};

pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let span = Span::current();
    let ip = (|| Some(req.headers().get("X-Forwarded-For")?.to_str().ok()?))().unwrap_or("undefined");
    info!("Received request from {} on: {:?}", ip, req.uri());
    let response = next.run(req).instrument(span).await;
    info!("Response status: {:?}", response.status());
    response
}

#[macro_export]
macro_rules! make_unique_span {
    ($name:ident) => {
        let id : $crate::uuid::Uuid = $crate::uuid::Uuid::new_v4();
        let $name = $crate::tracing::info_span!("", %id);
    };

    ($prefix:expr, $name:ident) => {
        let id : $crate::uuid::Uuid = $crate::uuid::Uuid::new_v4();
        let $name = $crate::tracing::info_span!($prefix, %id);
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
