//! This contains the health server, a basic server that for now always returns 200.
//! This is necessary to run the processor in Cloud Run, which expects to be able to
//! query a HTTP server to check for liveness.

use anyhow::Result;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};
use tracing::info;

/// This configures the health server.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HealthServerConfig {
    pub listen_address: SocketAddrV4,
}

impl Default for HealthServerConfig {
    fn default() -> Self {
        Self {
            listen_address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080),
        }
    }
}

pub async fn run(config: HealthServerConfig) -> Result<()> {
    info!("Health server starting at {}", config.listen_address);
    let app = Router::new().route("/", get(|| async { "Healthy!" }));

    let listener = tokio::net::TcpListener::bind(config.listen_address)
        .await
        .unwrap();

    eprintln!("Running server on port {}", config.listen_address);

    axum::serve(listener, app).await.unwrap();

    Err(anyhow::anyhow!("Server stopped running"))
}
