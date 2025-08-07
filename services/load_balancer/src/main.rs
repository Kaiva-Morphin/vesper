
use async_trait::async_trait;
use pingora::{http::ResponseHeader, listeners::tls::TlsSettings, prelude::*, server::configuration::ServerConf};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;

use tracing::info;
use tracing_log::LogTracer;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer};


fn main() -> anyhow::Result<()> {
    LogTracer::init()?;
    let subscriber = tracing_subscriber::registry()
        .with(fmt::layer().with_filter(LevelFilter::INFO));
    tracing::subscriber::set_global_default(subscriber).ok();
    
    CryptoProvider::install_default(default_provider()).ok();
    let mut server = Server::new_with_opt_and_conf(None, ServerConf{
        // todo: dockerize
        grace_period_seconds: Some(u64::MAX),
        graceful_shutdown_timeout_seconds: Some(u64::MAX),
        ..Default::default()
    });
    server.bootstrap();
    
    

    let cert_path = format!("{}/../../certs/http/fullchain.pem", env!("CARGO_MANIFEST_DIR"));
    let key_path = format!("{}/../../certs/http/privkey.pem", env!("CARGO_MANIFEST_DIR"));

    let mut tls_settings = TlsSettings::intermediate(&cert_path, &key_path).unwrap();
    tls_settings.enable_h2();

    
    let mut proxy = http_proxy_service(
        &server.configuration,
        Gateway {
        },
    );
    proxy.add_tls_with_settings("0.0.0.0:443", None, tls_settings);
    server.add_service(proxy);
    server.run_forever();

}

pub struct Gateway {

}

shared::env_config!(
    ".env" => ENV = Env {
        MY_IP : String
    }
);

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        let addr = session.client_addr().cloned().unwrap().as_inet().unwrap().ip().to_string();
        upstream_request
            .insert_header("X-Forwarded-For", addr.to_string())
            .unwrap();
        upstream_request
            .insert_header("X-Real-Ip", addr.to_string())
            .unwrap();
        info!("Headers for {addr} set!");
        upstream_request
            .insert_header("Host", "one.one.one.one")
            .unwrap();
        Ok(())
    }
    async fn request_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();
        let addr = session.client_addr().unwrap().as_inet().unwrap().ip().to_string();

        if addr != "127.0.0.1" && addr != ENV.MY_IP {return Err(pingora::Error::new_str("Unknown addr!"))}
        let addr = 
        if path.starts_with("/api/auth") {("127.0.0.1", 16090)}
        else if path.starts_with("/assets") {("127.0.0.1", 5000)}
        else {("127.0.0.1", 12345)};
        info!("proxying to {addr:?}");
        let peer = Box::new(HttpPeer::new(addr, false, "127.0.0.1".to_string()));
        Ok(peer)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());
        info!(
            "{} response code: {response_code}",
            self.request_summary(session, ctx)
        );
    }
}
