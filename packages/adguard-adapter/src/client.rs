use reqwest::{header, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use url::Url;

use crate::{
    DnsInfo, Operation, QueryLog, QueryLogInfo, ServerStatus, Stats, StatsInfo, TlsStatus,
};

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AdguardAdapterError {
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
pub struct AdguardAdapter {
    http: reqwest::Client,
    base_url: Url,
    credentials: Option<Credentials>,
}

impl AdguardAdapter {
    pub fn new(base_url: Url, credentials: Option<Credentials>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("nabe-adguard-adapter/0.0.0")
            .build()
            .expect("reqwest client config must be valid");

        Self {
            http,
            base_url,
            credentials,
        }
    }

    pub async fn status(&self) -> Result<ServerStatus, AdguardAdapterError> {
        self.get_json("status").await
    }

    pub async fn dns_info(&self) -> Result<DnsInfo, AdguardAdapterError> {
        self.get_json("dns_info").await
    }

    pub async fn querylog_info(&self) -> Result<QueryLogInfo, AdguardAdapterError> {
        self.get_json("querylog_info").await
    }

    pub async fn stats_info(&self) -> Result<StatsInfo, AdguardAdapterError> {
        self.get_json("stats_info").await
    }

    pub async fn stats(&self) -> Result<Stats, AdguardAdapterError> {
        self.get_json("stats").await
    }

    pub async fn tls_status(&self) -> Result<TlsStatus, AdguardAdapterError> {
        self.get_json("tls/status").await
    }

    pub async fn querylog(&self) -> Result<QueryLog, AdguardAdapterError> {
        self.get_json("querylog").await
    }

    pub async fn raw_get(&self, path: &str) -> Result<serde_json::Value, AdguardAdapterError> {
        self.get_json(path).await
    }

    pub async fn raw_get_with_query(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<serde_json::Value, AdguardAdapterError> {
        let mut url = self.control_url(path)?;
        url.query_pairs_mut()
            .extend_pairs(query.iter().map(|(key, value)| (*key, value.as_str())));
        self.request_url::<serde_json::Value>(url, reqwest::Method::GET, None)
            .await
    }

    pub async fn raw_json<B: Serialize + ?Sized>(
        &self,
        operation: Operation,
        body: Option<&B>,
    ) -> Result<serde_json::Value, AdguardAdapterError> {
        self.request(operation.path, operation.method.into(), body)
            .await
    }

    pub async fn post_json<B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<serde_json::Value, AdguardAdapterError> {
        self.request(path, reqwest::Method::POST, Some(body)).await
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, AdguardAdapterError> {
        self.request::<serde_json::Value>(path, reqwest::Method::GET, None)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|error| {
                    tracing::warn!(?error, %path, "failed to decode AdGuard response");
                    AdguardAdapterError::BadResponse
                })
            })
    }

    async fn request<B: Serialize + ?Sized>(
        &self,
        path: &str,
        method: reqwest::Method,
        body: Option<&B>,
    ) -> Result<serde_json::Value, AdguardAdapterError> {
        let url = self.control_url(path)?;
        self.request_url(url, method, body).await
    }

    async fn request_url<B: Serialize + ?Sized>(
        &self,
        url: Url,
        method: reqwest::Method,
        body: Option<&B>,
    ) -> Result<serde_json::Value, AdguardAdapterError> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(AdguardAdapterError::NotConfigured)?;
        let mut request = self.http.request(method, url);
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request
            .basic_auth(&credentials.username, Some(&credentials.password))
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(?error, "AdGuard request failed");
                AdguardAdapterError::Unreachable
            })?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(AdguardAdapterError::AuthFailed)
            }
            status if !status.is_success() => {
                tracing::warn!(%status, "AdGuard returned non-success status");
                return Err(AdguardAdapterError::BadResponse);
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
                AdguardAdapterError::BadResponse
            })
        } else {
            let text = response
                .text()
                .await
                .map_err(|_| AdguardAdapterError::BadResponse)?;
            Ok(serde_json::json!({ "ok": true, "body": text }))
        }
    }

    fn control_url(&self, path: &str) -> Result<Url, AdguardAdapterError> {
        let normalized = path.trim_start_matches('/');
        self.base_url
            .join(&format!("control/{normalized}"))
            .map_err(|error| {
                tracing::warn!(?error, %path, "invalid AdGuard path");
                AdguardAdapterError::BadResponse
            })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use url::Url;

    use crate::{AdguardAdapter, Credentials, Stats};

    #[test]
    fn stats_decode_uses_adguard_summary_fields() {
        let stats: Stats = serde_json::from_value(json!({
            "dns_queries": [0, 2],
            "blocked_filtering": [0, 1],
            "num_dns_queries": 2,
            "num_blocked_filtering": 1
        }))
        .expect("stats");

        assert_eq!(stats.num_dns_queries, Some(2));
        assert_eq!(stats.num_blocked_filtering, Some(1));
    }

    #[test]
    fn control_url_is_based_on_control_prefix() {
        let adapter = AdguardAdapter::new(
            Url::parse("http://dns-adguard:3000").unwrap(),
            Some(Credentials::new("admin", "test")),
        );

        let url = adapter.control_url("stats").expect("url");
        assert_eq!(url.as_str(), "http://dns-adguard:3000/control/stats");
    }
}
