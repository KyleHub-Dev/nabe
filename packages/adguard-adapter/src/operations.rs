#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
}

impl From<HttpMethod> for reqwest::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Operation {
    pub id: &'static str,
    pub method: HttpMethod,
    pub path: &'static str,
}

impl Operation {
    pub const fn new(id: &'static str, method: HttpMethod, path: &'static str) -> Self {
        Self { id, method, path }
    }
}

pub const ALL_OPERATIONS: &[Operation] = &[
    Operation::new("status", HttpMethod::Get, "status"),
    Operation::new("dnsInfo", HttpMethod::Get, "dns_info"),
    Operation::new("dnsConfig", HttpMethod::Post, "dns_config"),
    Operation::new("setProtection", HttpMethod::Post, "protection"),
    Operation::new("cacheClear", HttpMethod::Post, "cache_clear"),
    Operation::new("testUpstreamDNS", HttpMethod::Post, "test_upstream_dns"),
    Operation::new("getVersionJson", HttpMethod::Post, "version.json"),
    Operation::new("beginUpdate", HttpMethod::Post, "update"),
    Operation::new("queryLog", HttpMethod::Get, "querylog"),
    Operation::new("queryLogInfo", HttpMethod::Get, "querylog_info"),
    Operation::new("queryLogConfig", HttpMethod::Post, "querylog_config"),
    Operation::new("querylogClear", HttpMethod::Post, "querylog_clear"),
    Operation::new("getQueryLogConfig", HttpMethod::Get, "querylog/config"),
    Operation::new(
        "putQueryLogConfig",
        HttpMethod::Put,
        "querylog/config/update",
    ),
    Operation::new("stats", HttpMethod::Get, "stats"),
    Operation::new("statsReset", HttpMethod::Post, "stats_reset"),
    Operation::new("statsInfo", HttpMethod::Get, "stats_info"),
    Operation::new("statsConfig", HttpMethod::Post, "stats_config"),
    Operation::new("getStatsConfig", HttpMethod::Get, "stats/config"),
    Operation::new("putStatsConfig", HttpMethod::Put, "stats/config/update"),
    Operation::new("tlsStatus", HttpMethod::Get, "tls/status"),
    Operation::new("tlsConfigure", HttpMethod::Post, "tls/configure"),
    Operation::new("tlsValidate", HttpMethod::Post, "tls/validate"),
    Operation::new("dhcpStatus", HttpMethod::Get, "dhcp/status"),
    Operation::new("dhcpInterfaces", HttpMethod::Get, "dhcp/interfaces"),
    Operation::new("dhcpSetConfig", HttpMethod::Post, "dhcp/set_config"),
    Operation::new("checkActiveDhcp", HttpMethod::Post, "dhcp/find_active_dhcp"),
    Operation::new(
        "dhcpAddStaticLease",
        HttpMethod::Post,
        "dhcp/add_static_lease",
    ),
    Operation::new(
        "dhcpRemoveStaticLease",
        HttpMethod::Post,
        "dhcp/remove_static_lease",
    ),
    Operation::new(
        "dhcpUpdateStaticLease",
        HttpMethod::Post,
        "dhcp/update_static_lease",
    ),
    Operation::new("dhcpReset", HttpMethod::Post, "dhcp/reset"),
    Operation::new("dhcpResetLeases", HttpMethod::Post, "dhcp/reset_leases"),
    Operation::new("filteringStatus", HttpMethod::Get, "filtering/status"),
    Operation::new("filteringConfig", HttpMethod::Post, "filtering/config"),
    Operation::new("filteringAddURL", HttpMethod::Post, "filtering/add_url"),
    Operation::new(
        "filteringRemoveURL",
        HttpMethod::Post,
        "filtering/remove_url",
    ),
    Operation::new("filteringSetURL", HttpMethod::Post, "filtering/set_url"),
    Operation::new("filteringRefresh", HttpMethod::Post, "filtering/refresh"),
    Operation::new("filteringSetRules", HttpMethod::Post, "filtering/set_rules"),
    Operation::new(
        "filteringCheckHost",
        HttpMethod::Get,
        "filtering/check_host",
    ),
    Operation::new(
        "safebrowsingEnable",
        HttpMethod::Post,
        "safebrowsing/enable",
    ),
    Operation::new(
        "safebrowsingDisable",
        HttpMethod::Post,
        "safebrowsing/disable",
    ),
    Operation::new("safebrowsingStatus", HttpMethod::Get, "safebrowsing/status"),
    Operation::new("parentalEnable", HttpMethod::Post, "parental/enable"),
    Operation::new("parentalDisable", HttpMethod::Post, "parental/disable"),
    Operation::new("parentalStatus", HttpMethod::Get, "parental/status"),
    Operation::new("safesearchEnable", HttpMethod::Post, "safesearch/enable"),
    Operation::new("safesearchDisable", HttpMethod::Post, "safesearch/disable"),
    Operation::new("safesearchSettings", HttpMethod::Put, "safesearch/settings"),
    Operation::new("safesearchStatus", HttpMethod::Get, "safesearch/status"),
    Operation::new("clientsStatus", HttpMethod::Get, "clients"),
    Operation::new("clientsAdd", HttpMethod::Post, "clients/add"),
    Operation::new("clientsDelete", HttpMethod::Post, "clients/delete"),
    Operation::new("clientsUpdate", HttpMethod::Post, "clients/update"),
    Operation::new("clientsFind", HttpMethod::Get, "clients/find"),
    Operation::new("clientsSearch", HttpMethod::Post, "clients/search"),
    Operation::new("accessList", HttpMethod::Get, "access/list"),
    Operation::new("accessSet", HttpMethod::Post, "access/set"),
    Operation::new(
        "blockedServicesAvailableServices",
        HttpMethod::Get,
        "blocked_services/services",
    ),
    Operation::new(
        "blockedServicesAll",
        HttpMethod::Get,
        "blocked_services/all",
    ),
    Operation::new(
        "blockedServicesList",
        HttpMethod::Get,
        "blocked_services/list",
    ),
    Operation::new(
        "blockedServicesSet",
        HttpMethod::Post,
        "blocked_services/set",
    ),
    Operation::new(
        "blockedServicesSchedule",
        HttpMethod::Get,
        "blocked_services/get",
    ),
    Operation::new(
        "blockedServicesScheduleUpdate",
        HttpMethod::Put,
        "blocked_services/update",
    ),
    Operation::new("rewriteList", HttpMethod::Get, "rewrite/list"),
    Operation::new("rewriteAdd", HttpMethod::Post, "rewrite/add"),
    Operation::new("rewriteDelete", HttpMethod::Post, "rewrite/delete"),
    Operation::new("rewriteSettingsGet", HttpMethod::Get, "rewrite/settings"),
    Operation::new(
        "rewriteSettingsUpdate",
        HttpMethod::Put,
        "rewrite/settings/update",
    ),
    Operation::new("rewriteUpdate", HttpMethod::Put, "rewrite/update"),
    Operation::new("changeLanguage", HttpMethod::Post, "i18n/change_language"),
    Operation::new("currentLanguage", HttpMethod::Get, "i18n/current_language"),
    Operation::new(
        "installGetAddresses",
        HttpMethod::Get,
        "install/get_addresses",
    ),
    Operation::new(
        "installCheckConfig",
        HttpMethod::Post,
        "install/check_config",
    ),
    Operation::new("installConfigure", HttpMethod::Post, "install/configure"),
    Operation::new("login", HttpMethod::Post, "login"),
    Operation::new("logout", HttpMethod::Get, "logout"),
    Operation::new("updateProfile", HttpMethod::Put, "profile/update"),
    Operation::new("getProfile", HttpMethod::Get, "profile"),
    Operation::new("mobileConfigDoH", HttpMethod::Get, "apple/doh.mobileconfig"),
    Operation::new("mobileConfigDoT", HttpMethod::Get, "apple/dot.mobileconfig"),
];

#[cfg(test)]
mod tests {
    use super::ALL_OPERATIONS;

    #[test]
    fn openapi_operation_surface_is_complete_for_current_reference() {
        assert_eq!(ALL_OPERATIONS.len(), 81);
        assert!(ALL_OPERATIONS
            .iter()
            .any(|operation| operation.id == "status"));
        assert!(ALL_OPERATIONS
            .iter()
            .any(|operation| operation.id == "clientsAdd"));
        assert!(ALL_OPERATIONS
            .iter()
            .any(|operation| operation.id == "mobileConfigDoT"));
    }
}
