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

pub async fn unique_span(req: Request<Body>, next: Next) -> Response {
    let id : uuid::Uuid = uuid::Uuid::new_v4();
    let span = info_span!("request ", %id);
    let response = next.run(req).instrument(span.clone()).await;
    response
}
