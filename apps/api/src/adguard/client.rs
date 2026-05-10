#![allow(dead_code)]

use reqwest::{header, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use url::Url;

use super::{DnsInfo, QueryLog, QueryLogInfo, ServerStatus, Stats, StatsInfo, TlsStatus};

#[derive(Debug, Error)]
pub enum AdguardError {
    #[error("AdGuard credentials are not configured")]
    NotConfigured,
    #[error("AdGuard authentication failed")]
    AuthFailed,
    #[error("AdGuard is unreachable")]
    Unreachable,
    #[error("AdGuard returned a bad response")]
    BadResponse,
}

#[derive(Clone)]
pub struct AdguardClient {
    http: reqwest::Client,
    base_url: Url,
    username: Option<String>,
    password: Option<String>,
}

impl AdguardClient {
    pub fn new(base_url: Url, username: Option<String>, password: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("nabe-api/0.0.0")
            .build()
            .expect("reqwest client config must be valid");

        Self {
            http,
            base_url,
            username,
            password,
        }
    }

    pub async fn get_status(&self) -> Result<ServerStatus, AdguardError> {
        self.get_json("status").await
    }

    pub async fn get_dns_info(&self) -> Result<DnsInfo, AdguardError> {
        self.get_json("dns_info").await
    }

    pub async fn get_querylog_info(&self) -> Result<QueryLogInfo, AdguardError> {
        self.get_json("querylog_info").await
    }

    pub async fn get_stats_info(&self) -> Result<StatsInfo, AdguardError> {
        self.get_json("stats_info").await
    }

    pub async fn get_stats(&self) -> Result<Stats, AdguardError> {
        self.get_json("stats").await
    }

    pub async fn get_tls_status(&self) -> Result<TlsStatus, AdguardError> {
        self.get_json("tls/status").await
    }

    pub async fn get_querylog(&self) -> Result<QueryLog, AdguardError> {
        self.get_json("querylog").await
    }

    pub async fn get_raw(&self, path: &str) -> Result<serde_json::Value, AdguardError> {
        self.get_json(path).await
    }

    pub async fn post_json<T: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<serde_json::Value, AdguardError> {
        self.request(path, Some(body)).await
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, AdguardError> {
        self.request::<serde_json::Value>(path, None::<&serde_json::Value>)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|error| {
                    tracing::warn!(?error, %path, "failed to decode AdGuard response");
                    AdguardError::BadResponse
                })
            })
    }

    async fn request<B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: Option<&B>,
    ) -> Result<serde_json::Value, AdguardError> {
        let (username, password) = match (&self.username, &self.password) {
            (Some(username), Some(password)) => (username, password),
            _ => return Err(AdguardError::NotConfigured),
        };

        let url = self.control_url(path)?;
        let mut request = if let Some(body) = body {
            self.http.post(url).json(body)
        } else {
            self.http.get(url)
        };

        request = request.basic_auth(username, Some(password));
        let response = request.send().await.map_err(|error| {
            tracing::warn!(?error, "AdGuard request failed");
            AdguardError::Unreachable
        })?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(AdguardError::AuthFailed)
            }
            status if !status.is_success() => {
                tracing::warn!(%status, "AdGuard returned non-success status");
                return Err(AdguardError::BadResponse);
            }
            _ => {}
        }

        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();

        if content_type.contains("application/json") {
            response.json().await.map_err(|error| {
                tracing::warn!(?error, "AdGuard JSON decode failed");
                AdguardError::BadResponse
            })
        } else {
            let text = response
                .text()
                .await
                .map_err(|_| AdguardError::BadResponse)?;
            Ok(serde_json::json!({ "ok": true, "body": text }))
        }
    }

    fn control_url(&self, path: &str) -> Result<Url, AdguardError> {
        let normalized = path.trim_start_matches('/');
        self.base_url
            .join(&format!("control/{normalized}"))
            .map_err(|error| {
                tracing::warn!(?error, %path, "invalid AdGuard path");
                AdguardError::BadResponse
            })
    }
}
