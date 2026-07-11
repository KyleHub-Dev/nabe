use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use nabe_adguard_adapter::AdguardAdapterError;

use crate::{
    auth::{
        self,
        authorization::{
            self, EDGE_MANAGE, EDGE_READ, PLATFORM_ADMIN, QUERYLOG_READ_TENANT, STATS_READ_TENANT,
        },
        session, Principal,
    },
    db::repositories::{
        DeviceClientInput, DeviceClientRecord, DnsPolicyInput, DnsPolicyRecord, EdgeNodeInput,
        EdgeNodeRecord,
    },
    error::ApiError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/auth/login", get(auth_login))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/logout", post(auth_logout).get(auth_logout))
        .route("/api/session", get(session_me))
        .route("/api/permissions/me", get(permissions_me))
        .route("/api/dashboard", get(dashboard))
        .route("/api/engine/status", get(engine_status))
        .route("/api/engine/adguard/status", get(adguard_status))
        .route("/api/engine/adguard/stats", get(adguard_stats))
        .route("/api/engine/adguard/querylog", get(adguard_querylog))
        .route("/api/dev/edge-token", get(dev_edge_token))
        .route("/api/edge/enroll", post(edge_enroll))
        .route("/api/edge/heartbeat", post(edge_heartbeat))
        .route("/api/edge/nodes", get(edge_nodes))
        .route(
            "/api/device-clients",
            get(device_clients_list).post(device_clients_create),
        )
        .route(
            "/api/device-clients/{id}",
            get(device_clients_get).put(device_clients_update),
        )
        .route(
            "/api/dns-policies",
            get(dns_policies_list).post(dns_policies_create),
        )
        .route(
            "/api/dns-policies/{id}",
            get(dns_policies_get).put(dns_policies_update),
        )
        .route("/api/dns-policies/apply", post(dns_policies_apply))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "nabe-api",
    })
}

#[derive(Serialize)]
struct ReadyResponse {
    status: &'static str,
    checks: ReadyChecks,
}

#[derive(Serialize)]
struct ReadyChecks {
    database: &'static str,
    adguard: &'static str,
}

async fn ready(State(state): State<AppState>) -> Json<ReadyResponse> {
    let database = if state.db.ping().await.is_ok() {
        "ok"
    } else {
        "error"
    };
    let adguard = match state.adguard.status().await {
        Ok(_) => "ok",
        Err(AdguardAdapterError::NotConfigured) => "not_configured",
        Err(AdguardAdapterError::AuthFailed) => "auth_failed",
        Err(AdguardAdapterError::Unreachable) => "error",
        Err(AdguardAdapterError::BadResponse) => "degraded",
    };

    let status = if database == "ok" && adguard == "ok" {
        "ok"
    } else {
        "degraded"
    };

    Json(ReadyResponse {
        status,
        checks: ReadyChecks { database, adguard },
    })
}

async fn auth_login(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let http = reqwest::Client::new();
    let (redirect, cookies) = auth::oidc::build_login_redirect(&state.config, &http).await?;
    Ok(with_set_cookie_headers(redirect, cookies))
}

#[derive(Deserialize)]
struct AuthCallbackQuery {
    code: String,
    state: String,
}

async fn auth_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthCallbackQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let transient = session::read_oidc_transient(&headers)?;
    if transient.state != query.state {
        return Err(ApiError::InvalidAuthState);
    }

    let http = reqwest::Client::new();
    let principal = auth::oidc::exchange_code_for_principal(
        &state.config,
        &http,
        &query.code,
        &transient.verifier,
    )
    .await?;
    let subject = auth::oidc::subject_input(&principal);
    state.db.upsert_subject(&subject).await?;
    tracing::info!(
        provider = %principal.provider,
        subject = %principal.subject,
        email = principal.email.as_deref().unwrap_or(""),
        display_name = principal.display_name.as_deref().unwrap_or(""),
        roles = ?principal.roles,
        "user authenticated"
    );

    let secure = state.config.public_url.scheme() == "https";
    let mut cookies = session::clear_oidc_cookies();
    cookies.push(session::create_session_cookie(
        &principal,
        &state.config.session_secret,
        secure,
    )?);

    Ok(with_set_cookie_headers(
        Redirect::to(state.config.public_url.as_str()),
        cookies,
    ))
}

async fn auth_logout(State(state): State<AppState>) -> impl IntoResponse {
    let response = Redirect::to(state.config.oidc_post_logout_redirect_uri.as_str());
    with_set_cookie_headers(response, vec![session::clear_session_cookie()])
}

async fn session_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let principal = session::read_principal(&headers, &state.config.session_secret)?;
    tracing::info!(
        provider = %principal.provider,
        subject = %principal.subject,
        email = principal.email.as_deref().unwrap_or(""),
        display_name = principal.display_name.as_deref().unwrap_or(""),
        roles = ?principal.roles,
        "session read"
    );
    Ok(Json(SessionResponse {
        authenticated: true,
        principal,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PermissionsResponse {
    principal: Principal,
    effective: authorization::EffectivePermissions,
}

async fn permissions_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PermissionsResponse>, ApiError> {
    let principal = session::read_principal(&headers, &state.config.session_secret)?;
    Ok(Json(PermissionsResponse {
        effective: authorization::effective_permissions(&principal),
        principal,
    }))
}

fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    permission: &'static str,
) -> Result<Principal, ApiError> {
    let principal = session::read_principal(headers, &state.config.session_secret)?;
    authorization::require(&principal, permission)?;
    Ok(principal)
}

fn audit_control_read(principal: &Principal, action: &'static str, resource: &str) {
    tracing::info!(
        action,
        resource,
        provider = %principal.provider,
        subject = %principal.subject,
        roles = ?principal.roles,
        outcome = "allowed",
        "control-plane read"
    );
}

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DashboardResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    tracing::info!(
        provider = %principal.provider,
        subject = %principal.subject,
        email = principal.email.as_deref().unwrap_or(""),
        display_name = principal.display_name.as_deref().unwrap_or(""),
        roles = ?principal.roles,
        "dashboard requested"
    );
    let engine = load_engine_status(&state).await?;
    let stats = match state.adguard.stats().await {
        Ok(stats) => DashboardStats {
            available: true,
            dns_queries: stats.num_dns_queries.unwrap_or_default(),
            blocked_filtering: stats.num_blocked_filtering.unwrap_or_default(),
        },
        Err(_) => DashboardStats {
            available: false,
            dns_queries: 0,
            blocked_filtering: 0,
        },
    };

    Ok(Json(DashboardResponse {
        principal,
        engine,
        stats,
        adguard: AdguardDebugAccess {
            url: state.config.adguard_debug_url.as_str().to_string(),
        },
    }))
}

async fn engine_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<EngineStatus>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    audit_control_read(&principal, "engine.status", "cloud-dns");
    Ok(Json(load_engine_status(&state).await?))
}

async fn adguard_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    audit_control_read(&principal, "engine.adguard.status", "cloud-dns");
    state
        .adguard
        .raw_get("status")
        .await
        .map(Json)
        .map_err(map_adguard_error)
}

async fn adguard_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let principal = authorize(&state, &headers, STATS_READ_TENANT)?;
    audit_control_read(&principal, "engine.adguard.stats", "cloud-dns");
    state
        .adguard
        .raw_get("stats")
        .await
        .map(Json)
        .map_err(map_adguard_error)
}

async fn adguard_querylog(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let principal = authorize(&state, &headers, QUERYLOG_READ_TENANT)?;
    audit_control_read(&principal, "engine.adguard.querylog", "cloud-dns");
    state
        .adguard
        .raw_get("querylog")
        .await
        .map(Json)
        .map_err(map_adguard_error)
}

async fn dev_edge_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DevEdgeTokenResponse>, ApiError> {
    let principal = authorize(&state, &headers, PLATFORM_ADMIN)?;
    audit_control_read(&principal, "dev.edge_token", "dev");
    Ok(Json(DevEdgeTokenResponse {
        configured: state.config.dev_edge_token.is_some(),
        token: state.config.dev_edge_token.clone(),
        warning: "dev-only bootstrap token; do not use for production enrollment",
    }))
}

async fn edge_nodes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<EdgeNodesResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    let config = state.config.clone();
    audit_control_read(&principal, "edge.nodes", "all");
    Ok(Json(EdgeNodesResponse {
        nodes: state
            .db
            .list_edge_nodes()
            .await?
            .into_iter()
            .map(|record| EdgeNodeResponse::from_record(record, &config))
            .collect(),
    }))
}

async fn edge_enroll(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EdgeNodeRequest>,
) -> Result<Json<EdgeNodeResponse>, ApiError> {
    verify_edge_token(&state, &headers)?;
    let node = upsert_edge_node(&state, request).await?;
    Ok(Json(EdgeNodeResponse::from_record(node, &state.config)))
}

async fn edge_heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EdgeNodeRequest>,
) -> Result<Json<EdgeNodeResponse>, ApiError> {
    verify_edge_token(&state, &headers)?;
    let node = upsert_edge_node(&state, request).await?;
    Ok(Json(EdgeNodeResponse::from_record(node, &state.config)))
}

async fn upsert_edge_node(
    state: &AppState,
    request: EdgeNodeRequest,
) -> Result<EdgeNodeRecord, ApiError> {
    if request.node_id.trim().is_empty()
        || request.node_name.trim().is_empty()
        || request.hostname.trim().is_empty()
    {
        return Err(ApiError::BadRequest);
    }

    let inventory_json = serde_json::to_string(&request.inventory).map_err(|error| {
        tracing::error!(?error, "edge node inventory serialization failed");
        ApiError::Internal
    })?;
    state
        .db
        .upsert_edge_node(&EdgeNodeInput {
            id: request.node_id,
            name: request.node_name,
            hostname: request.hostname,
            architecture: request.architecture,
            os: request.os,
            kernel: request.kernel,
            speiche_version: request.speiche_version,
            health_status: request.health_status,
            inventory_json: Some(inventory_json),
        })
        .await
}

fn audit_control_write(principal: &Principal, action: &'static str, resource: &str) {
    tracing::info!(
        action,
        resource,
        provider = %principal.provider,
        subject = %principal.subject,
        roles = ?principal.roles,
        outcome = "allowed",
        "control-plane write"
    );
}

async fn device_clients_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DeviceClientsResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    audit_control_read(&principal, "device_clients.list", "all");
    Ok(Json(DeviceClientsResponse {
        clients: state
            .db
            .list_device_clients()
            .await?
            .into_iter()
            .map(DeviceClientResponse::from_record)
            .collect(),
    }))
}

async fn device_clients_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DeviceClientResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    audit_control_read(&principal, "device_clients.get", &id);
    Ok(Json(DeviceClientResponse::from_record(
        state.db.device_client(&id).await?,
    )))
}

async fn device_clients_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DeviceClientCreateRequest>,
) -> Result<Json<DeviceClientResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE)?;
    let label = validate_label(&request.label)?;
    let client_id = validate_client_id(&request.client_id)?;
    if state
        .db
        .device_client_by_client_id(&client_id)
        .await?
        .is_some()
    {
        return Err(ApiError::BadRequest);
    }
    let input = DeviceClientInput {
        id: uuid::Uuid::new_v4().to_string(),
        label,
        owner_scope: normalize_owner_scope(request.owner_scope.as_deref()),
        client_id,
        enabled: request.enabled.unwrap_or(true),
    };
    audit_control_write(&principal, "device_clients.create", &input.id);
    Ok(Json(DeviceClientResponse::from_record(
        state.db.create_device_client(&input).await?,
    )))
}

async fn device_clients_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<DeviceClientUpdateRequest>,
) -> Result<Json<DeviceClientResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE)?;
    let existing = state.db.device_client(&id).await?;
    let label = match request.label {
        Some(value) => validate_label(&value)?,
        None => existing.label,
    };
    let client_id = match request.client_id {
        Some(value) => validate_client_id(&value)?,
        None => existing.client_id.clone(),
    };
    if client_id != existing.client_id {
        if let Some(other) = state.db.device_client_by_client_id(&client_id).await? {
            if other.id != id {
                return Err(ApiError::BadRequest);
            }
        }
    }
    let input = DeviceClientInput {
        id: id.clone(),
        label,
        owner_scope: match request.owner_scope {
            Some(value) => normalize_owner_scope(Some(&value)),
            None => existing.owner_scope,
        },
        client_id,
        enabled: request.enabled.unwrap_or(existing.enabled),
    };
    audit_control_write(&principal, "device_clients.update", &id);
    Ok(Json(DeviceClientResponse::from_record(
        state.db.update_device_client(&input).await?,
    )))
}

async fn dns_policies_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DnsPoliciesResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    audit_control_read(&principal, "dns_policies.list", "all");
    Ok(Json(DnsPoliciesResponse {
        policies: state
            .db
            .list_dns_policies()
            .await?
            .into_iter()
            .map(DnsPolicyResponse::from_record)
            .collect(),
    }))
}

async fn dns_policies_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DnsPolicyResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ)?;
    audit_control_read(&principal, "dns_policies.get", &id);
    Ok(Json(DnsPolicyResponse::from_record(
        state.db.dns_policy(&id).await?,
    )))
}

async fn dns_policies_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DnsPolicyCreateRequest>,
) -> Result<Json<DnsPolicyResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE)?;
    let name = validate_label(&request.name)?;
    let blocked_domains = validate_blocked_domains(&request.blocked_domains)?;
    let input = DnsPolicyInput {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        blocked_domains_json: serde_json::to_string(&blocked_domains)
            .map_err(|_| ApiError::Internal)?,
        enabled: request.enabled.unwrap_or(true),
    };
    audit_control_write(&principal, "dns_policies.create", &input.id);
    Ok(Json(DnsPolicyResponse::from_record(
        state.db.create_dns_policy(&input).await?,
    )))
}

async fn dns_policies_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<DnsPolicyUpdateRequest>,
) -> Result<Json<DnsPolicyResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE)?;
    let existing = state.db.dns_policy(&id).await?;
    let name = match request.name {
        Some(value) => validate_label(&value)?,
        None => existing.name,
    };
    let blocked_domains_json = match request.blocked_domains {
        Some(domains) => serde_json::to_string(&validate_blocked_domains(&domains)?)
            .map_err(|_| ApiError::Internal)?,
        None => existing.blocked_domains_json,
    };
    let input = DnsPolicyInput {
        id: id.clone(),
        name,
        blocked_domains_json,
        enabled: request.enabled.unwrap_or(existing.enabled),
    };
    audit_control_write(&principal, "dns_policies.update", &id);
    Ok(Json(DnsPolicyResponse::from_record(
        state.db.update_dns_policy(&input).await?,
    )))
}

async fn dns_policies_apply(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DnsPolicyApplyResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE)?;
    audit_control_write(&principal, "dns_policies.apply", "cloud-dns");

    let policies: Vec<DnsPolicyRecord> = state
        .db
        .list_dns_policies()
        .await?
        .into_iter()
        .filter(|policy| policy.enabled)
        .collect();
    let active_client_ids: Vec<String> = state
        .db
        .list_device_clients()
        .await?
        .into_iter()
        .filter(|client| client.enabled)
        .map(|client| client.client_id)
        .collect();
    let rules = compose_block_rules(&policies)?;

    state
        .adguard
        .post_json(
            "filtering/set_rules",
            &serde_json::json!({ "rules": rules }),
        )
        .await
        .map_err(map_adguard_error)?;

    let applied_policy_ids: Vec<String> = policies.into_iter().map(|policy| policy.id).collect();
    state
        .db
        .mark_dns_policies_applied(&applied_policy_ids)
        .await?;

    Ok(Json(DnsPolicyApplyResponse {
        applied_policy_ids,
        rules,
        active_client_ids,
        applied_at: now_rfc3339(),
    }))
}

fn compose_block_rules(policies: &[DnsPolicyRecord]) -> Result<Vec<String>, ApiError> {
    let mut domains = Vec::new();
    for policy in policies {
        let parsed: Vec<String> =
            serde_json::from_str(&policy.blocked_domains_json).map_err(|error| {
                tracing::error!(?error, policy_id = %policy.id, "stored blocked domains are invalid");
                ApiError::Internal
            })?;
        domains.extend(parsed);
    }
    domains.sort_unstable();
    domains.dedup();
    Ok(domains
        .into_iter()
        .map(|domain| format!("||{domain}^"))
        .collect())
}

fn validate_label(value: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 120 {
        return Err(ApiError::BadRequest);
    }
    Ok(trimmed.to_string())
}

fn normalize_owner_scope(value: Option<&str>) -> String {
    let trimmed = value.unwrap_or("").trim();
    if trimmed.is_empty() {
        "default".to_string()
    } else {
        trimmed.to_string()
    }
}

fn validate_client_id(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_ascii_lowercase();
    let valid_length = (1..=64).contains(&normalized.len());
    let valid_chars = normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid_length || !valid_chars || normalized.starts_with('-') || normalized.ends_with('-') {
        return Err(ApiError::BadRequest);
    }
    Ok(normalized)
}

fn validate_blocked_domains(domains: &[String]) -> Result<Vec<String>, ApiError> {
    if domains.is_empty() {
        return Err(ApiError::BadRequest);
    }
    let mut normalized = Vec::new();
    for domain in domains {
        normalized.push(validate_domain(domain)?);
    }
    normalized.sort_unstable();
    normalized.dedup();
    Ok(normalized)
}

fn validate_domain(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().trim_end_matches('.').to_ascii_lowercase();
    if normalized.is_empty() || normalized.len() > 253 || !normalized.contains('.') {
        return Err(ApiError::BadRequest);
    }
    for segment in normalized.split('.') {
        let valid_length = (1..=63).contains(&segment.len());
        let valid_chars = segment
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !valid_length || !valid_chars || segment.starts_with('-') || segment.ends_with('-') {
            return Err(ApiError::BadRequest);
        }
    }
    Ok(normalized)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn verify_edge_token(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = state
        .config
        .dev_edge_token
        .as_deref()
        .ok_or(ApiError::Forbidden)?;
    let provided = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .or_else(|| {
            headers
                .get("x-nabe-edge-token")
                .and_then(|value| value.to_str().ok())
        })
        .unwrap_or_default();

    if provided == expected {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    authenticated: bool,
    principal: Principal,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineStatus {
    kind: &'static str,
    name: &'static str,
    connected: bool,
    version: Option<String>,
    protection_enabled: bool,
    query_log_enabled: bool,
    statistics_enabled: bool,
    last_checked_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardResponse {
    principal: Principal,
    engine: EngineStatus,
    stats: DashboardStats,
    adguard: AdguardDebugAccess,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardStats {
    available: bool,
    dns_queries: u64,
    blocked_filtering: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdguardDebugAccess {
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DevEdgeTokenResponse {
    configured: bool,
    token: Option<String>,
    warning: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EdgeNodeRequest {
    node_id: String,
    node_name: String,
    hostname: String,
    architecture: String,
    os: String,
    kernel: String,
    speiche_version: String,
    health_status: String,
    inventory: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceClientCreateRequest {
    label: String,
    owner_scope: Option<String>,
    client_id: String,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceClientUpdateRequest {
    label: Option<String>,
    owner_scope: Option<String>,
    client_id: Option<String>,
    enabled: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceClientsResponse {
    clients: Vec<DeviceClientResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceClientResponse {
    id: String,
    label: String,
    owner_scope: String,
    client_id: String,
    enabled: bool,
    created_at: String,
    updated_at: String,
}

impl DeviceClientResponse {
    fn from_record(record: DeviceClientRecord) -> Self {
        Self {
            id: record.id,
            label: record.label,
            owner_scope: record.owner_scope,
            client_id: record.client_id,
            enabled: record.enabled,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DnsPolicyCreateRequest {
    name: String,
    blocked_domains: Vec<String>,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DnsPolicyUpdateRequest {
    name: Option<String>,
    blocked_domains: Option<Vec<String>>,
    enabled: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DnsPoliciesResponse {
    policies: Vec<DnsPolicyResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DnsPolicyResponse {
    id: String,
    name: String,
    blocked_domains: Vec<String>,
    enabled: bool,
    last_applied_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl DnsPolicyResponse {
    fn from_record(record: DnsPolicyRecord) -> Self {
        let blocked_domains =
            serde_json::from_str(&record.blocked_domains_json).unwrap_or_default();
        Self {
            id: record.id,
            name: record.name,
            blocked_domains,
            enabled: record.enabled,
            last_applied_at: record.last_applied_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DnsPolicyApplyResponse {
    applied_policy_ids: Vec<String>,
    rules: Vec<String>,
    active_client_ids: Vec<String>,
    applied_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeNodesResponse {
    nodes: Vec<EdgeNodeResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeNodeResponse {
    node_id: String,
    node_name: String,
    hostname: String,
    architecture: String,
    os: String,
    kernel: String,
    speiche_version: String,
    health_status: String,
    inventory: Option<serde_json::Value>,
    enrolled_at: String,
    last_seen_at: String,
}

impl EdgeNodeResponse {
    fn from_record(record: EdgeNodeRecord, config: &crate::config::Config) -> Self {
        let inventory = record
            .inventory_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok());
        let health_status = effective_edge_health_status(&record, config);
        Self {
            node_id: record.id,
            node_name: record.name,
            hostname: record.hostname,
            architecture: record.architecture,
            os: record.os,
            kernel: record.kernel,
            speiche_version: record.speiche_version,
            health_status,
            inventory,
            enrolled_at: record.enrolled_at,
            last_seen_at: record.last_seen_at,
        }
    }
}

fn effective_edge_health_status(record: &EdgeNodeRecord, config: &crate::config::Config) -> String {
    let Ok(last_seen_at) = OffsetDateTime::parse(
        &record.last_seen_at,
        &time::format_description::well_known::Rfc3339,
    ) else {
        return "stale".to_string();
    };
    let age = OffsetDateTime::now_utc() - last_seen_at;
    let heartbeat = config.edge_heartbeat_interval_seconds.max(5);
    let stale_after = heartbeat * config.edge_stale_after_intervals.max(1);
    let offline_after = heartbeat * config.edge_offline_after_intervals.max(1);

    if age >= Duration::seconds(offline_after) {
        "offline".to_string()
    } else if age >= Duration::seconds(stale_after) {
        "stale".to_string()
    } else {
        record.health_status.clone()
    }
}

async fn load_engine_status(state: &AppState) -> Result<EngineStatus, ApiError> {
    let status = state.adguard.status().await.map_err(map_adguard_error)?;
    let querylog = state.adguard.querylog_info().await.ok();
    let stats = state.adguard.stats_info().await.ok();
    let statistics_enabled = stats.is_some();

    Ok(EngineStatus {
        kind: "adguard-home",
        name: "Cloud DNS",
        connected: true,
        version: status.version,
        protection_enabled: status.protection_enabled.unwrap_or(false),
        query_log_enabled: querylog.and_then(|value| value.enabled).unwrap_or(false),
        statistics_enabled,
        last_checked_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    })
}

fn map_adguard_error(error: AdguardAdapterError) -> ApiError {
    match error {
        AdguardAdapterError::NotConfigured => ApiError::AdguardNotConfigured,
        AdguardAdapterError::AuthFailed => ApiError::AdguardAuthFailed,
        AdguardAdapterError::Unreachable => ApiError::AdguardUnreachable,
        AdguardAdapterError::BadResponse => ApiError::BadUpstreamResponse,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compose_block_rules, validate_blocked_domains, validate_client_id, validate_domain,
        validate_label,
    };
    use crate::db::repositories::DnsPolicyRecord;

    fn policy(id: &str, domains: &str, enabled: bool) -> DnsPolicyRecord {
        DnsPolicyRecord {
            id: id.to_string(),
            name: format!("policy {id}"),
            blocked_domains_json: domains.to_string(),
            enabled,
            last_applied_at: None,
            created_at: "2026-07-11T00:00:00Z".to_string(),
            updated_at: "2026-07-11T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn compose_block_rules_is_deterministic_and_deduplicated() {
        let policies = vec![
            policy("a", r#"["blocked.nabe.test","ads.example.com"]"#, true),
            policy("b", r#"["blocked.nabe.test"]"#, true),
        ];
        let rules = compose_block_rules(&policies).expect("rules");
        assert_eq!(
            rules,
            vec![
                "||ads.example.com^".to_string(),
                "||blocked.nabe.test^".to_string()
            ]
        );
    }

    #[test]
    fn compose_block_rules_rejects_corrupt_stored_domains() {
        let policies = vec![policy("a", "not-json", true)];
        assert!(compose_block_rules(&policies).is_err());
    }

    #[test]
    fn domain_validation_normalizes_and_rejects_invalid_names() {
        assert_eq!(
            validate_domain(" Blocked.Nabe.Test. ").expect("domain"),
            "blocked.nabe.test"
        );
        assert!(validate_domain("no-dot").is_err());
        assert!(validate_domain("bad domain.example").is_err());
        assert!(validate_domain("-leading.example.com").is_err());
        assert!(validate_domain("").is_err());
    }

    #[test]
    fn blocked_domains_require_at_least_one_entry() {
        assert!(validate_blocked_domains(&[]).is_err());
        let normalized = validate_blocked_domains(&[
            "blocked.nabe.test".to_string(),
            "BLOCKED.nabe.test".to_string(),
        ])
        .expect("domains");
        assert_eq!(normalized, vec!["blocked.nabe.test".to_string()]);
    }

    #[test]
    fn client_id_validation_enforces_adguard_clientid_charset() {
        assert_eq!(validate_client_id(" Pixel-8 ").expect("id"), "pixel-8");
        assert!(validate_client_id("bad_id").is_err());
        assert!(validate_client_id("-bad").is_err());
        assert!(validate_client_id("").is_err());
    }

    #[test]
    fn label_validation_trims_and_rejects_empty() {
        assert_eq!(
            validate_label(" Living Room TV ").expect("label"),
            "Living Room TV"
        );
        assert!(validate_label("   ").is_err());
    }
}

fn with_set_cookie_headers<T: IntoResponse>(
    response: T,
    cookies: Vec<String>,
) -> impl IntoResponse {
    let mut response = response.into_response();
    for cookie in cookies {
        if let Ok(value) = HeaderValue::from_str(&cookie) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    (StatusCode::FOUND, response)
}
