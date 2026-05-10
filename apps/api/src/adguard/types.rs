#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub version: Option<String>,
    #[serde(default)]
    pub protection_enabled: Option<bool>,
    #[serde(default)]
    pub dns_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsInfo {
    #[serde(default)]
    pub protection_enabled: Option<bool>,
    #[serde(default)]
    pub upstream_dns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogInfo {
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsInfo {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub dns_queries: Option<u64>,
    #[serde(default)]
    pub blocked_filtering: Option<u64>,
    #[serde(default)]
    pub replaced_safebrowsing: Option<u64>,
    #[serde(default)]
    pub replaced_parental: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsStatus {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLog {
    #[serde(default)]
    pub data: serde_json::Value,
}
