use std::env;

use anyhow::{anyhow, Context};
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,
    pub public_url: Url,
    pub web_origin: String,
    pub database_path: String,
    pub session_secret: String,
    pub adguard_base_url: Url,
    pub adguard_debug_url: Url,
    pub adguard_username: Option<String>,
    pub adguard_password: Option<String>,
    pub dev_edge_token: Option<String>,
    pub edge_heartbeat_interval_seconds: i64,
    pub edge_stale_after_intervals: i64,
    pub edge_offline_after_intervals: i64,
    pub oidc_issuer_url: Url,
    pub oidc_client_id: Option<String>,
    pub oidc_redirect_uri: Url,
    pub oidc_post_logout_redirect_uri: Url,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let public_url = parse_url("NABE_PUBLIC_URL", "http://localhost:5173")?;
        let oidc_redirect_uri =
            parse_url("OIDC_REDIRECT_URI", "http://localhost:8080/auth/callback")?;
        let oidc_post_logout_redirect_uri =
            parse_url("OIDC_POST_LOGOUT_REDIRECT_URI", "http://localhost:5173/")?;

        Ok(Self {
            api_host: env_or("NABE_API_HOST", "0.0.0.0"),
            api_port: env_or("NABE_API_PORT", "8080")
                .parse()
                .context("NABE_API_PORT must be a valid port")?,
            web_origin: origin(&public_url)?,
            public_url,
            database_path: env_or("NABE_DATABASE_PATH", "/data/nabe.db"),
            session_secret: env_or("SESSION_SECRET", "change-me-dev-only"),
            adguard_base_url: parse_url("ADGUARD_BASE_URL", "http://dns-adguard:3000")?,
            adguard_debug_url: parse_url("ADGUARD_DEBUG_URL", "http://localhost:3000")?,
            adguard_username: env_opt("ADGUARD_USERNAME"),
            adguard_password: env_opt("ADGUARD_PASSWORD"),
            dev_edge_token: env_opt("NABE_DEV_EDGE_TOKEN"),
            edge_heartbeat_interval_seconds: env_i64("NABE_EDGE_HEARTBEAT_INTERVAL_SECONDS", 30)?,
            edge_stale_after_intervals: env_i64("NABE_EDGE_STALE_AFTER_INTERVALS", 3)?,
            edge_offline_after_intervals: env_i64("NABE_EDGE_OFFLINE_AFTER_INTERVALS", 10)?,
            oidc_issuer_url: parse_url("OIDC_ISSUER_URL", "https://auth.kylehub.dev")?,
            oidc_client_id: env_opt("OIDC_CLIENT_ID"),
            oidc_redirect_uri,
            oidc_post_logout_redirect_uri,
        })
    }
}

fn env_or(key: &str, fallback: &str) -> String {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn env_opt(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

fn env_i64(key: &str, fallback: i64) -> anyhow::Result<i64> {
    env_or(key, &fallback.to_string())
        .parse()
        .with_context(|| format!("{key} must be an integer"))
}

fn parse_url(key: &str, fallback: &str) -> anyhow::Result<Url> {
    Url::parse(&env_or(key, fallback)).with_context(|| format!("{key} must be a valid URL"))
}

fn origin(url: &Url) -> anyhow::Result<String> {
    let scheme = url.scheme();
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("NABE_PUBLIC_URL must include a host"))?;
    let port = url
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    Ok(format!("{scheme}://{host}{port}"))
}
