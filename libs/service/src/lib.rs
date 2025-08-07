use axum::Router;
use anyhow::{Error, Result};
use tracing::{info, warn};


pub struct Service {
    router: Option<Router>
}


impl Service {
    pub fn begin() -> Self {
        shared::utils::logger::init_logger();
        Service { router: None }
    }
    pub fn route(
        &mut self,
        router: Router
    ) {
        self.router = Some(router);
    }
    pub async fn run(
        self,
        port: u16
    ) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

        let v = listener.local_addr();
        if let Ok(a) = v {
            info!("Listening on {}", a);
        } else {
            warn!("Failed to get local address");
        }
        let Some(router) = self.router else {return Err(Error::msg("Nothing to serve!"))};
        axum::serve(listener, router.into_make_service()).await?;
        Ok(())
    }
    pub async fn run_with_connect_info(
        self,
        port: u16
    ) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

        let v = listener.local_addr();
        if let Ok(a) = v {
            info!("Listening on {}", a);
        } else {
            warn!("Failed to get local address");
        }
        let Some(router) = self.router else {return Err(Error::msg("Nothing to serve!"))};
        axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
        Ok(())
    }
}
