#[derive(Debug, Clone)]
pub struct SubjectInput {
    pub id: String,
    pub provider: String,
    pub subject: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantAuthorization {
    pub tenant_id: String,
    pub tenant_slug: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationRecord {
    pub global_roles: Vec<String>,
    pub global_permissions: Vec<String>,
    pub tenants: Vec<TenantAuthorization>,
}

#[derive(Debug, Clone)]
pub struct TenantRecord {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct DnsStatBucketInput {
    pub device_client_id: String,
    pub bucket_start: String,
    pub bucket_seconds: i64,
    pub queries: i64,
    pub blocked: i64,
    pub cached: i64,
}

#[derive(Debug, Clone)]
pub struct DnsStatBucketRecord {
    pub edge_node_id: String,
    pub device_client_id: String,
    pub bucket_start: String,
    pub bucket_seconds: i64,
    pub queries: i64,
    pub blocked: i64,
    pub cached: i64,
}

#[derive(Debug, Clone)]
pub struct AuditEventInput {
    pub actor_provider: Option<String>,
    pub actor_subject: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub tenant_id: Option<String>,
    pub outcome: String,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditEventRecord {
    pub id: String,
    pub actor_provider: Option<String>,
    pub actor_subject: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub tenant_id: Option<String>,
    pub outcome: String,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Default, Clone)]
pub struct AuditEventFilter {
    pub actor_subject: Option<String>,
    pub action: Option<String>,
    pub tenant_id: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone)]
pub struct EdgeNodeInput {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub architecture: String,
    pub os: String,
    pub kernel: String,
    pub speiche_version: String,
    pub health_status: String,
    pub inventory_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceClientInput {
    pub id: String,
    pub label: String,
    pub owner_scope: String,
    pub owner_provider: Option<String>,
    pub owner_subject: Option<String>,
    pub tenant_id: Option<String>,
    pub client_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct DeviceClientRecord {
    pub id: String,
    pub label: String,
    pub owner_scope: String,
    pub owner_provider: Option<String>,
    pub owner_subject: Option<String>,
    pub tenant_id: Option<String>,
    pub client_id: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct DnsPolicyInput {
    pub id: String,
    pub name: String,
    pub blocked_domains_json: String,
    pub enabled: bool,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DnsPolicyRecord {
    pub id: String,
    pub name: String,
    pub blocked_domains_json: String,
    pub enabled: bool,
    pub tenant_id: Option<String>,
    pub last_applied_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct EdgeNodeRecord {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub architecture: String,
    pub os: String,
    pub kernel: String,
    pub speiche_version: String,
    pub health_status: String,
    pub inventory_json: Option<String>,
    pub enrolled_at: String,
    pub last_seen_at: String,
}
