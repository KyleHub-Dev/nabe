mod auth;
mod config;
mod db;
mod error;
mod http;
mod state;

use std::net::SocketAddr;

use anyhow::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::Config, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nabe_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = Config::from_env()?;
    let state = AppState::new(config.clone()).await?;
    let app = http::router(state);
    let addr: SocketAddr = format!("{}:{}", config.api_host, config.api_port)
        .parse()
        .context("invalid NABE_API_HOST/NABE_API_PORT")?;

    tracing::info!(%addr, "nabe api starting");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
