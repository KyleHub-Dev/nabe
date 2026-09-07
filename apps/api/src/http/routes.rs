use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

use nabe_adguard_adapter::AdguardAdapterError;

use crate::{
    auth::{
        self,
        authorization::{self, AUDIT_READ, EDGE_MANAGE, EDGE_READ, PLATFORM_ADMIN},
        session, Principal,
    },
    db::repositories::{
        AuditEventFilter, AuditEventInput, AuditEventRecord, DeviceClientInput, DeviceClientRecord,
        DnsPolicyInput, DnsPolicyRecord, DnsStatBucketInput, DnsStatBucketRecord, EdgeNodeInput,
        EdgeNodeRecord, TenantRecord,
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
        .route("/api/audit/events", get(audit_events_list))
        .route("/api/tenants", get(tenants_list).post(tenants_create))
        .route("/api/tenants/{id}/members", post(tenant_members_grant))
        .route("/api/dashboard", get(dashboard))
        .route("/api/engine/status", get(engine_status))
        .route("/api/engine/adguard/status", get(adguard_status))
        .route("/api/engine/adguard/stats", get(adguard_stats))
        .route("/api/engine/adguard/querylog", get(adguard_querylog))
        .route("/api/dev/edge-token", get(dev_edge_token))
        .route("/api/edge/enroll", post(edge_enroll))
        .route("/api/edge/heartbeat", post(edge_heartbeat))
        .route("/api/edge/stats", post(edge_stats_ingest))
        .route("/api/edge/nodes", get(edge_nodes))
        .route("/api/edge/nodes/{id}/reenroll", post(edge_reenroll))
        .route("/api/stats/history", get(stats_history))
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
    let bootstrap_key = format!("{}:{}", principal.provider, principal.subject);
    if state
        .config
        .bootstrap_admin_subjects
        .iter()
        .any(|candidate| candidate == &bootstrap_key)
    {
        state
            .db
            .grant_global_role(&principal.provider, &principal.subject, "admin")
            .await?;
    }
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
    let identity = session::read_principal(&headers, &state.config.session_secret)?;
    let effective = resolve_permissions(&state, &identity).await?;
    let principal = authorization::principal_with_roles(identity, &effective);
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
    let identity = session::read_principal(&headers, &state.config.session_secret)?;
    let effective = resolve_permissions(&state, &identity).await?;
    let principal = authorization::principal_with_roles(identity, &effective);
    Ok(Json(PermissionsResponse {
        effective,
        principal,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditEventsQuery {
    actor_subject: Option<String>,
    action: Option<String>,
    tenant_id: Option<String>,
    limit: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEventsResponse {
    events: Vec<AuditEventResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEventResponse {
    id: String,
    actor_provider: Option<String>,
    actor_subject: Option<String>,
    action: String,
    target_type: String,
    target_id: Option<String>,
    tenant_id: Option<String>,
    outcome: String,
    reason: Option<String>,
    request_id: Option<String>,
    metadata: Option<serde_json::Value>,
    created_at: String,
}

impl AuditEventResponse {
    fn from_record(record: AuditEventRecord) -> Self {
        Self {
            id: record.id,
            actor_provider: record.actor_provider,
            actor_subject: record.actor_subject,
            action: record.action,
            target_type: record.target_type,
            target_id: record.target_id,
            tenant_id: record.tenant_id,
            outcome: record.outcome,
            reason: record.reason,
            request_id: record.request_id,
            metadata: record
                .metadata_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            created_at: record.created_at,
        }
    }
}

async fn audit_events_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditEventsQuery>,
) -> Result<Json<AuditEventsResponse>, ApiError> {
    let principal = authorize(&state, &headers, AUDIT_READ).await?;
    let filter = AuditEventFilter {
        actor_subject: query.actor_subject,
        action: query.action,
        tenant_id: query.tenant_id,
        limit: query.limit.unwrap_or(100),
    };
    persist_audit(&state, &principal, "audit.events.list", "audit-event", None).await?;
    Ok(Json(AuditEventsResponse {
        events: state
            .db
            .list_audit_events(&filter)
            .await?
            .into_iter()
            .map(AuditEventResponse::from_record)
            .collect(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TenantResponse {
    id: String,
    slug: String,
    name: String,
    created_at: String,
    updated_at: String,
}

impl From<TenantRecord> for TenantResponse {
    fn from(tenant: TenantRecord) -> Self {
        Self {
            id: tenant.id,
            slug: tenant.slug,
            name: tenant.name,
            created_at: tenant.created_at,
            updated_at: tenant.updated_at,
        }
    }
}

#[derive(Serialize)]
struct TenantsResponse {
    tenants: Vec<TenantResponse>,
}

#[derive(Deserialize)]
struct TenantCreateRequest {
    slug: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TenantMemberGrantRequest {
    provider: String,
    subject: String,
    role: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TenantMemberGrantResponse {
    tenant_id: String,
    provider: String,
    subject: String,
    role: String,
}

async fn authorization_context(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(Principal, authorization::EffectivePermissions), ApiError> {
    let identity = session::read_principal(headers, &state.config.session_secret)?;
    let effective = resolve_permissions(state, &identity).await?;
    let principal = authorization::principal_with_roles(identity, &effective);
    Ok((principal, effective))
}

async fn tenants_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TenantsResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let tenants = if authorization::require_global(&effective, PLATFORM_ADMIN).is_ok() {
        state.db.list_tenants().await?
    } else {
        let mut tenants = Vec::new();
        for grant in &effective.tenants {
            tenants.push(state.db.tenant(&grant.tenant_id).await?);
        }
        tenants
    };
    audit_control_read(&state, &principal, "tenants.list", "authorized").await?;
    Ok(Json(TenantsResponse {
        tenants: tenants.into_iter().map(TenantResponse::from).collect(),
    }))
}

async fn tenants_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TenantCreateRequest>,
) -> Result<Json<TenantResponse>, ApiError> {
    let principal = authorize(&state, &headers, PLATFORM_ADMIN).await?;
    let slug = validate_tenant_slug(&request.slug)?;
    let name = validate_label(&request.name)?;
    let tenant = state
        .db
        .create_tenant(&uuid::Uuid::new_v4().to_string(), &slug, &name)
        .await?;
    audit_control_write(&state, &principal, "tenants.create", &tenant.id).await?;
    Ok(Json(TenantResponse::from(tenant)))
}

async fn tenant_members_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
    Json(request): Json<TenantMemberGrantRequest>,
) -> Result<Json<TenantMemberGrantResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    if authorization::require_global(&effective, PLATFORM_ADMIN).is_err() {
        authorization::require_tenant(&effective, &tenant_id, "tenant.manage")?;
    }
    let role = match request.role.as_str() {
        "manager" | "user" | "viewer" => request.role,
        _ => return Err(ApiError::BadRequest),
    };
    if request.provider.trim().is_empty() || request.subject.trim().is_empty() {
        return Err(ApiError::BadRequest);
    }
    state
        .db
        .grant_tenant_role(
            &tenant_id,
            request.provider.trim(),
            request.subject.trim(),
            &role,
            None,
        )
        .await?;
    audit_control_write(&state, &principal, "tenant.members.grant", &tenant_id).await?;
    Ok(Json(TenantMemberGrantResponse {
        tenant_id,
        provider: request.provider.trim().to_string(),
        subject: request.subject.trim().to_string(),
        role,
    }))
}

async fn resolve_permissions(
    state: &AppState,
    principal: &Principal,
) -> Result<authorization::EffectivePermissions, ApiError> {
    Ok(state
        .db
        .authorization_for_subject(&principal.provider, &principal.subject)
        .await?
        .into())
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    permission: &'static str,
) -> Result<Principal, ApiError> {
    let identity = session::read_principal(headers, &state.config.session_secret)?;
    let effective = resolve_permissions(state, &identity).await?;
    authorization::require_global(&effective, permission)?;
    Ok(authorization::principal_with_roles(identity, &effective))
}

async fn persist_audit(
    state: &AppState,
    principal: &Principal,
    action: &'static str,
    target_type: &'static str,
    target_id: Option<&str>,
) -> Result<(), ApiError> {
    state
        .db
        .append_audit_event(&AuditEventInput {
            actor_provider: Some(principal.provider.clone()),
            actor_subject: Some(principal.subject.clone()),
            action: action.to_string(),
            target_type: target_type.to_string(),
            target_id: target_id.map(str::to_string),
            tenant_id: None,
            outcome: "allowed".to_string(),
            reason: None,
            request_id: None,
            metadata_json: None,
        })
        .await?;
    Ok(())
}

async fn audit_control_read(
    state: &AppState,
    principal: &Principal,
    action: &'static str,
    resource: &str,
) -> Result<(), ApiError> {
    tracing::info!(
        action,
        resource,
        provider = %principal.provider,
        subject = %principal.subject,
        roles = ?principal.roles,
        outcome = "allowed",
        "control-plane read"
    );
    persist_audit(state, principal, action, "control-plane", Some(resource)).await
}

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DashboardResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ).await?;
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
    let principal = authorize(&state, &headers, EDGE_READ).await?;
    audit_control_read(&state, &principal, "engine.status", "cloud-dns").await?;
    Ok(Json(load_engine_status(&state).await?))
}

async fn adguard_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_READ).await?;
    audit_control_read(&state, &principal, "engine.adguard.status", "cloud-dns").await?;
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
    Query(query): Query<QueryLogQuery>,
) -> Result<Json<FilteredStatsResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let reason = validate_audit_reason(&query.reason)?;
    let since = parse_querylog_since(query.since.as_deref())?;
    let clients = authorized_stats_clients(
        &state,
        &principal,
        &effective,
        query.tenant_id.as_deref(),
        query.device_client_id.as_deref(),
    )
    .await?;
    let upstream = state
        .adguard
        .raw_get_with_query("querylog", &[("limit", "5000".to_string())])
        .await
        .map_err(map_adguard_error)?;
    let response = aggregate_authorized_stats(&upstream, &clients, since);
    state
        .db
        .append_audit_event(&AuditEventInput {
            actor_provider: Some(principal.provider.clone()),
            actor_subject: Some(principal.subject.clone()),
            action: "stats.read".to_string(),
            target_type: "dns-statistics".to_string(),
            target_id: query.device_client_id,
            tenant_id: query.tenant_id,
            outcome: "allowed".to_string(),
            reason: Some(reason),
            request_id: None,
            metadata_json: Some(
                serde_json::json!({
                    "windowStart": response.window_start,
                    "clientIds": response.client_ids,
                    "queries": response.queries,
                    "blocked": response.blocked
                })
                .to_string(),
            ),
        })
        .await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatsHistoryQuery {
    tenant_id: Option<String>,
    device_client_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoricalStatBucketResponse {
    edge_node_id: String,
    device_client_id: String,
    bucket_start: String,
    bucket_seconds: i64,
    queries: i64,
    blocked: i64,
    cached: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatsHistoryResponse {
    from: String,
    to: String,
    buckets: Vec<HistoricalStatBucketResponse>,
}

async fn stats_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<StatsHistoryQuery>,
) -> Result<Json<StatsHistoryResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let reason = validate_audit_reason(&query.reason)?;
    let now = OffsetDateTime::now_utc();
    let to = query
        .to
        .as_deref()
        .map(parse_rfc3339)
        .transpose()?
        .unwrap_or(now);
    let from = query
        .from
        .as_deref()
        .map(parse_rfc3339)
        .transpose()?
        .unwrap_or(to - Duration::days(1));
    if to > now + Duration::minutes(5)
        || from >= to
        || from < now - Duration::days(state.config.stats_retention_days)
    {
        return Err(ApiError::BadRequest);
    }
    let clients = authorized_stats_clients(
        &state,
        &principal,
        &effective,
        query.tenant_id.as_deref(),
        query.device_client_id.as_deref(),
    )
    .await?;
    let allowed: std::collections::HashSet<String> =
        clients.into_iter().map(|client| client.id).collect();
    let from_text = format_rfc3339(from)?;
    let to_text = format_rfc3339(to)?;
    let buckets: Vec<HistoricalStatBucketResponse> = state
        .db
        .list_dns_stat_buckets(&from_text, &to_text)
        .await?
        .into_iter()
        .filter(|bucket| allowed.contains(&bucket.device_client_id))
        .map(HistoricalStatBucketResponse::from_record)
        .collect();
    state
        .db
        .append_audit_event(&AuditEventInput {
            actor_provider: Some(principal.provider.clone()),
            actor_subject: Some(principal.subject.clone()),
            action: "stats.history.read".to_string(),
            target_type: "dns-stat-buckets".to_string(),
            target_id: query.device_client_id,
            tenant_id: query.tenant_id,
            outcome: "allowed".to_string(),
            reason: Some(reason),
            request_id: None,
            metadata_json: Some(
                serde_json::json!({
                    "from": from_text,
                    "to": to_text,
                    "bucketCount": buckets.len()
                })
                .to_string(),
            ),
        })
        .await?;
    Ok(Json(StatsHistoryResponse {
        from: from_text,
        to: to_text,
        buckets,
    }))
}

impl HistoricalStatBucketResponse {
    fn from_record(record: DnsStatBucketRecord) -> Self {
        Self {
            edge_node_id: record.edge_node_id,
            device_client_id: record.device_client_id,
            bucket_start: record.bucket_start,
            bucket_seconds: record.bucket_seconds,
            queries: record.queries,
            blocked: record.blocked,
            cached: record.cached,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryLogQuery {
    tenant_id: Option<String>,
    device_client_id: Option<String>,
    since: Option<String>,
    limit: Option<usize>,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilteredQueryLogResponse {
    window_start: String,
    client_ids: Vec<String>,
    returned: usize,
    data: Vec<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceStatsResponse {
    device_client_id: String,
    client_id: String,
    queries: u64,
    blocked: u64,
    cached: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilteredStatsResponse {
    window_start: String,
    client_ids: Vec<String>,
    queries: u64,
    blocked: u64,
    cached: u64,
    truncated: bool,
    devices: Vec<DeviceStatsResponse>,
}

async fn adguard_querylog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<QueryLogQuery>,
) -> Result<Json<FilteredQueryLogResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let reason = validate_audit_reason(&query.reason)?;
    let since = parse_querylog_since(query.since.as_deref())?;
    let limit = query.limit.unwrap_or(100);
    if !(1..=500).contains(&limit) {
        return Err(ApiError::BadRequest);
    }
    let clients = authorized_querylog_clients(
        &state,
        &principal,
        &effective,
        query.tenant_id.as_deref(),
        query.device_client_id.as_deref(),
    )
    .await?;
    let allowed_ids: std::collections::HashSet<String> = clients
        .iter()
        .map(|client| client.client_id.clone())
        .collect();
    let upstream = state
        .adguard
        .raw_get_with_query("querylog", &[("limit", "5000".to_string())])
        .await
        .map_err(map_adguard_error)?;
    let data = filter_querylog_entries(&upstream, &allowed_ids, since, limit);
    let window_start = since
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| ApiError::Internal)?;
    let mut client_ids: Vec<String> = allowed_ids.into_iter().collect();
    client_ids.sort();
    state
        .db
        .append_audit_event(&AuditEventInput {
            actor_provider: Some(principal.provider.clone()),
            actor_subject: Some(principal.subject.clone()),
            action: "querylog.read".to_string(),
            target_type: "dns-query-log".to_string(),
            target_id: query.device_client_id,
            tenant_id: query.tenant_id,
            outcome: "allowed".to_string(),
            reason: Some(reason),
            request_id: None,
            metadata_json: Some(
                serde_json::json!({
                    "windowStart": window_start,
                    "clientIds": client_ids,
                    "returned": data.len()
                })
                .to_string(),
            ),
        })
        .await?;
    Ok(Json(FilteredQueryLogResponse {
        window_start,
        client_ids,
        returned: data.len(),
        data,
    }))
}

async fn dev_edge_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DevEdgeTokenResponse>, ApiError> {
    let principal = authorize(&state, &headers, PLATFORM_ADMIN).await?;
    audit_control_read(&state, &principal, "dev.edge_token", "dev").await?;
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
    let principal = authorize(&state, &headers, EDGE_READ).await?;
    let config = state.config.clone();
    audit_control_read(&state, &principal, "edge.nodes", "all").await?;
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
    verify_enrollment_token(&state, &headers)?;
    let input = edge_node_input(request)?;
    let credential = generate_edge_credential();
    let node = state
        .db
        .enroll_edge_node(&input, &hash_edge_credential(&credential))
        .await?;
    let mut response = EdgeNodeResponse::from_record(node, &state.config);
    response.credential = Some(credential);
    Ok(Json(response))
}

async fn edge_heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EdgeNodeRequest>,
) -> Result<Json<EdgeNodeResponse>, ApiError> {
    verify_edge_credential(&state, &headers, &request.node_id).await?;
    let node = upsert_edge_node(&state, request).await?;
    Ok(Json(EdgeNodeResponse::from_record(node, &state.config)))
}

async fn edge_stats_ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EdgeStatsIngestRequest>,
) -> Result<Json<EdgeStatsIngestResponse>, ApiError> {
    verify_edge_credential(&state, &headers, &request.node_id).await?;
    validate_batch_id(&request.batch_id)?;
    state.db.edge_node(&request.node_id).await?;
    if request.buckets.len() > 1000 {
        return Err(ApiError::BadRequest);
    }
    let now = OffsetDateTime::now_utc();
    let retention_cutoff = now - Duration::days(state.config.stats_retention_days);
    let collected_through = parse_rfc3339(&request.collected_through)?;
    if collected_through > now + Duration::minutes(5) || collected_through < retention_cutoff {
        return Err(ApiError::BadRequest);
    }
    let mut buckets = Vec::with_capacity(request.buckets.len());
    for bucket in &request.buckets {
        let bucket_start = parse_rfc3339(&bucket.bucket_start)?;
        if bucket.bucket_seconds != 300
            || bucket_start.unix_timestamp().rem_euclid(300) != 0
            || bucket_start > collected_through
            || bucket_start < retention_cutoff
            || bucket.blocked > bucket.queries
            || bucket.cached > bucket.queries
            || bucket.queries > i64::MAX as u64
        {
            return Err(ApiError::BadRequest);
        }
        let Some(client) = state
            .db
            .device_client_by_client_id(&bucket.client_id)
            .await?
        else {
            continue;
        };
        buckets.push(DnsStatBucketInput {
            device_client_id: client.id,
            bucket_start: format_rfc3339(bucket_start)?,
            bucket_seconds: bucket.bucket_seconds,
            queries: bucket.queries as i64,
            blocked: bucket.blocked as i64,
            cached: bucket.cached as i64,
        });
    }
    let accepted = state
        .db
        .ingest_dns_stat_buckets(
            &request.batch_id,
            &request.node_id,
            &format_rfc3339(collected_through)?,
            &buckets,
            &format_rfc3339(retention_cutoff)?,
        )
        .await?;
    Ok(Json(EdgeStatsIngestResponse {
        accepted,
        batch_id: request.batch_id,
        bucket_count: buckets.len(),
    }))
}

async fn edge_reenroll(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(node_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE).await?;
    state.db.edge_node(&node_id).await?;
    state.db.revoke_edge_credential(&node_id).await?;
    audit_control_write(&state, &principal, "edge.credential.revoke", &node_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upsert_edge_node(
    state: &AppState,
    request: EdgeNodeRequest,
) -> Result<EdgeNodeRecord, ApiError> {
    let input = edge_node_input(request)?;
    state.db.upsert_edge_node(&input).await
}

fn edge_node_input(request: EdgeNodeRequest) -> Result<EdgeNodeInput, ApiError> {
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
    Ok(EdgeNodeInput {
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
}

async fn audit_control_write(
    state: &AppState,
    principal: &Principal,
    action: &'static str,
    resource: &str,
) -> Result<(), ApiError> {
    tracing::info!(
        action,
        resource,
        provider = %principal.provider,
        subject = %principal.subject,
        roles = ?principal.roles,
        outcome = "allowed",
        "control-plane write"
    );
    persist_audit(state, principal, action, "control-plane", Some(resource)).await
}

async fn device_clients_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DeviceClientsResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    audit_control_read(&state, &principal, "device_clients.list", "all").await?;
    Ok(Json(DeviceClientsResponse {
        clients: state
            .db
            .list_device_clients()
            .await?
            .into_iter()
            .filter(|client| can_read_device_client(&principal, &effective, client))
            .map(DeviceClientResponse::from_record)
            .collect(),
    }))
}

async fn device_clients_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DeviceClientResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let client = state.db.device_client(&id).await?;
    if !can_read_device_client(&principal, &effective, &client) {
        return Err(ApiError::NotFound);
    }
    audit_control_read(&state, &principal, "device_clients.get", &id).await?;
    Ok(Json(DeviceClientResponse::from_record(client)))
}

async fn device_clients_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DeviceClientCreateRequest>,
) -> Result<Json<DeviceClientResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    authorize_device_client_create(&effective, request.tenant_id.as_deref())?;
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
        owner_scope: request
            .tenant_id
            .as_deref()
            .map(|id| format!("tenant:{id}"))
            .unwrap_or_else(|| format!("subject:{}:{}", principal.provider, principal.subject)),
        owner_provider: Some(principal.provider.clone()),
        owner_subject: Some(principal.subject.clone()),
        tenant_id: request.tenant_id,
        client_id,
        enabled: request.enabled.unwrap_or(true),
    };
    audit_control_write(&state, &principal, "device_clients.create", &input.id).await?;
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
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let existing = state.db.device_client(&id).await?;
    if !can_manage_device_client(&principal, &effective, &existing) {
        return Err(ApiError::NotFound);
    }
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
        owner_scope: existing.owner_scope,
        owner_provider: existing.owner_provider,
        owner_subject: existing.owner_subject,
        tenant_id: existing.tenant_id,
        client_id,
        enabled: request.enabled.unwrap_or(existing.enabled),
    };
    audit_control_write(&state, &principal, "device_clients.update", &id).await?;
    Ok(Json(DeviceClientResponse::from_record(
        state.db.update_device_client(&input).await?,
    )))
}

async fn dns_policies_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DnsPoliciesResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    audit_control_read(&state, &principal, "dns_policies.list", "all").await?;
    Ok(Json(DnsPoliciesResponse {
        policies: state
            .db
            .list_dns_policies()
            .await?
            .into_iter()
            .filter(|policy| can_read_policy(&effective, policy))
            .map(DnsPolicyResponse::from_record)
            .collect(),
    }))
}

async fn dns_policies_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DnsPolicyResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let policy = state.db.dns_policy(&id).await?;
    if !can_read_policy(&effective, &policy) {
        return Err(ApiError::NotFound);
    }
    audit_control_read(&state, &principal, "dns_policies.get", &id).await?;
    Ok(Json(DnsPolicyResponse::from_record(policy)))
}

async fn dns_policies_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DnsPolicyCreateRequest>,
) -> Result<Json<DnsPolicyResponse>, ApiError> {
    let (principal, effective) = authorization_context(&state, &headers).await?;
    authorize_policy_manage(&effective, request.tenant_id.as_deref())?;
    let name = validate_label(&request.name)?;
    let blocked_domains = validate_blocked_domains(&request.blocked_domains)?;
    let input = DnsPolicyInput {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        blocked_domains_json: serde_json::to_string(&blocked_domains)
            .map_err(|_| ApiError::Internal)?,
        enabled: request.enabled.unwrap_or(true),
        tenant_id: request.tenant_id,
    };
    audit_control_write(&state, &principal, "dns_policies.create", &input.id).await?;
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
    let (principal, effective) = authorization_context(&state, &headers).await?;
    let existing = state.db.dns_policy(&id).await?;
    if !can_manage_policy(&effective, &existing) {
        return Err(ApiError::NotFound);
    }
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
        tenant_id: existing.tenant_id,
    };
    audit_control_write(&state, &principal, "dns_policies.update", &id).await?;
    Ok(Json(DnsPolicyResponse::from_record(
        state.db.update_dns_policy(&input).await?,
    )))
}

async fn dns_policies_apply(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DnsPolicyApplyResponse>, ApiError> {
    let principal = authorize(&state, &headers, EDGE_MANAGE).await?;
    audit_control_write(&state, &principal, "dns_policies.apply", "cloud-dns").await?;

    let policies: Vec<DnsPolicyRecord> = state
        .db
        .list_dns_policies()
        .await?
        .into_iter()
        .filter(|policy| policy.enabled)
        .collect();
    let active_clients: Vec<DeviceClientRecord> = state
        .db
        .list_device_clients()
        .await?
        .into_iter()
        .filter(|client| client.enabled)
        .collect();
    let rules = compose_block_rules(&policies, &active_clients)?;
    let active_client_ids = active_clients
        .iter()
        .map(|client| client.client_id.clone())
        .collect();

    reconcile_adguard_clients(&state, &active_clients).await?;

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

fn compose_block_rules(
    policies: &[DnsPolicyRecord],
    clients: &[DeviceClientRecord],
) -> Result<Vec<String>, ApiError> {
    let mut rules = Vec::new();
    for policy in policies {
        let parsed: Vec<String> =
            serde_json::from_str(&policy.blocked_domains_json).map_err(|error| {
                tracing::error!(?error, policy_id = %policy.id, "stored blocked domains are invalid");
                ApiError::Internal
            })?;
        match policy.tenant_id.as_deref() {
            None => rules.extend(parsed.into_iter().map(|domain| format!("||{domain}^"))),
            Some(tenant_id) => {
                for client in clients
                    .iter()
                    .filter(|client| client.tenant_id.as_deref() == Some(tenant_id))
                {
                    rules.extend(parsed.iter().map(|domain| {
                        format!(
                            "||{domain}^$client={}",
                            engine_client_name(&client.client_id)
                        )
                    }));
                }
            }
        }
    }
    rules.sort_unstable();
    rules.dedup();
    Ok(rules)
}

fn engine_client_name(client_id: &str) -> String {
    format!("nabe-{client_id}")
}

fn adguard_client_payload(client: &DeviceClientRecord) -> serde_json::Value {
    serde_json::json!({
        "name": engine_client_name(&client.client_id),
        "ids": [client.client_id],
        "tags": [],
        "use_global_settings": true,
        "filtering_enabled": true,
        "parental_enabled": false,
        "safebrowsing_enabled": false,
        "safesearch_enabled": false,
        "use_global_blocked_services": true,
        "blocked_services": [],
        "ignore_querylog": false,
        "ignore_statistics": false
    })
}

async fn reconcile_adguard_clients(
    state: &AppState,
    active_clients: &[DeviceClientRecord],
) -> Result<(), ApiError> {
    let status = state
        .adguard
        .raw_get("clients")
        .await
        .map_err(map_adguard_error)?;
    let existing_names: std::collections::HashSet<String> = status
        .get("clients")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|client| client.get("name").and_then(serde_json::Value::as_str))
        .filter(|name| name.starts_with("nabe-"))
        .map(str::to_string)
        .collect();
    let desired_names: std::collections::HashSet<String> = active_clients
        .iter()
        .map(|client| engine_client_name(&client.client_id))
        .collect();

    for client in active_clients {
        let name = engine_client_name(&client.client_id);
        let data = adguard_client_payload(client);
        let (path, payload) = if existing_names.contains(&name) {
            (
                "clients/update",
                serde_json::json!({ "name": name, "data": data }),
            )
        } else {
            ("clients/add", data)
        };
        state
            .adguard
            .post_json(path, &payload)
            .await
            .map_err(map_adguard_error)?;
    }

    for name in existing_names.difference(&desired_names) {
        state
            .adguard
            .post_json("clients/delete", &serde_json::json!({ "name": name }))
            .await
            .map_err(map_adguard_error)?;
    }
    Ok(())
}

fn validate_label(value: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 120 {
        return Err(ApiError::BadRequest);
    }
    Ok(trimmed.to_string())
}

fn validate_tenant_slug(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.len() > 63
        || normalized.starts_with('-')
        || normalized.ends_with('-')
        || !normalized
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ApiError::BadRequest);
    }
    Ok(normalized)
}

fn is_platform_admin(effective: &authorization::EffectivePermissions) -> bool {
    authorization::require_global(effective, PLATFORM_ADMIN).is_ok()
}

fn has_global(effective: &authorization::EffectivePermissions, permission: &str) -> bool {
    authorization::require_global(effective, permission).is_ok()
}

fn has_tenant(
    effective: &authorization::EffectivePermissions,
    tenant_id: &str,
    permission: &str,
) -> bool {
    authorization::require_tenant(effective, tenant_id, permission).is_ok()
}

fn authorize_device_client_create(
    effective: &authorization::EffectivePermissions,
    tenant_id: Option<&str>,
) -> Result<(), ApiError> {
    match tenant_id {
        Some(tenant_id) if is_platform_admin(effective) => {
            let _ = tenant_id;
            Ok(())
        }
        Some(tenant_id) => {
            authorization::require_tenant(effective, tenant_id, "tenant.device_client.manage")
        }
        None => authorization::require_global(effective, "device_client.manage_own"),
    }
}

fn can_read_device_client(
    principal: &Principal,
    effective: &authorization::EffectivePermissions,
    client: &DeviceClientRecord,
) -> bool {
    is_platform_admin(effective)
        || (client.owner_provider.as_deref() == Some(principal.provider.as_str())
            && client.owner_subject.as_deref() == Some(principal.subject.as_str())
            && has_global(effective, "device_client.read_own"))
        || client
            .tenant_id
            .as_deref()
            .is_some_and(|tenant_id| has_tenant(effective, tenant_id, "tenant.device_client.read"))
}

fn can_manage_device_client(
    principal: &Principal,
    effective: &authorization::EffectivePermissions,
    client: &DeviceClientRecord,
) -> bool {
    is_platform_admin(effective)
        || (client.owner_provider.as_deref() == Some(principal.provider.as_str())
            && client.owner_subject.as_deref() == Some(principal.subject.as_str())
            && has_global(effective, "device_client.manage_own"))
        || client.tenant_id.as_deref().is_some_and(|tenant_id| {
            has_tenant(effective, tenant_id, "tenant.device_client.manage")
        })
}

fn authorize_policy_manage(
    effective: &authorization::EffectivePermissions,
    tenant_id: Option<&str>,
) -> Result<(), ApiError> {
    match tenant_id {
        Some(tenant_id) if !is_platform_admin(effective) => {
            authorization::require_tenant(effective, tenant_id, "tenant.policy.manage")
        }
        Some(_) | None if is_platform_admin(effective) => Ok(()),
        None => Err(ApiError::Forbidden),
        Some(_) => unreachable!(),
    }
}

fn can_read_policy(
    effective: &authorization::EffectivePermissions,
    policy: &DnsPolicyRecord,
) -> bool {
    is_platform_admin(effective)
        || policy
            .tenant_id
            .as_deref()
            .is_some_and(|tenant_id| has_tenant(effective, tenant_id, "tenant.policy.read"))
}

fn can_manage_policy(
    effective: &authorization::EffectivePermissions,
    policy: &DnsPolicyRecord,
) -> bool {
    is_platform_admin(effective)
        || policy
            .tenant_id
            .as_deref()
            .is_some_and(|tenant_id| has_tenant(effective, tenant_id, "tenant.policy.manage"))
}

fn validate_audit_reason(value: &str) -> Result<String, ApiError> {
    let reason = value.trim();
    if reason.is_empty() || reason.len() > 200 {
        return Err(ApiError::BadRequest);
    }
    Ok(reason.to_string())
}

fn parse_querylog_since(value: Option<&str>) -> Result<OffsetDateTime, ApiError> {
    let now = OffsetDateTime::now_utc();
    let since = match value {
        Some(value) => OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
            .map_err(|_| ApiError::BadRequest)?,
        None => now - Duration::hours(1),
    };
    if since > now || since < now - Duration::hours(24) {
        return Err(ApiError::BadRequest);
    }
    Ok(since)
}

fn can_read_querylog_for_client(
    principal: &Principal,
    effective: &authorization::EffectivePermissions,
    client: &DeviceClientRecord,
) -> bool {
    is_platform_admin(effective)
        || (client.owner_provider.as_deref() == Some(principal.provider.as_str())
            && client.owner_subject.as_deref() == Some(principal.subject.as_str())
            && has_global(effective, "querylog.read_own"))
        || client
            .tenant_id
            .as_deref()
            .is_some_and(|tenant_id| has_tenant(effective, tenant_id, "tenant.querylog.read"))
}

async fn authorized_querylog_clients(
    state: &AppState,
    principal: &Principal,
    effective: &authorization::EffectivePermissions,
    tenant_id: Option<&str>,
    device_client_id: Option<&str>,
) -> Result<Vec<DeviceClientRecord>, ApiError> {
    if let Some(tenant_id) = tenant_id {
        if !is_platform_admin(effective)
            && !has_tenant(effective, tenant_id, "tenant.querylog.read")
        {
            return Err(ApiError::Forbidden);
        }
    }

    if let Some(device_client_id) = device_client_id {
        let client = state.db.device_client(device_client_id).await?;
        if tenant_id.is_some() && client.tenant_id.as_deref() != tenant_id {
            return Err(ApiError::NotFound);
        }
        if !can_read_querylog_for_client(principal, effective, &client) {
            return Err(ApiError::NotFound);
        }
        return Ok(vec![client]);
    }

    Ok(state
        .db
        .list_device_clients()
        .await?
        .into_iter()
        .filter(|client| tenant_id.is_none() || client.tenant_id.as_deref() == tenant_id)
        .filter(|client| can_read_querylog_for_client(principal, effective, client))
        .collect())
}

fn can_read_stats_for_client(
    principal: &Principal,
    effective: &authorization::EffectivePermissions,
    client: &DeviceClientRecord,
) -> bool {
    is_platform_admin(effective)
        || (client.owner_provider.as_deref() == Some(principal.provider.as_str())
            && client.owner_subject.as_deref() == Some(principal.subject.as_str())
            && has_global(effective, "stats.read_own"))
        || client
            .tenant_id
            .as_deref()
            .is_some_and(|tenant_id| has_tenant(effective, tenant_id, "tenant.stats.read"))
}

async fn authorized_stats_clients(
    state: &AppState,
    principal: &Principal,
    effective: &authorization::EffectivePermissions,
    tenant_id: Option<&str>,
    device_client_id: Option<&str>,
) -> Result<Vec<DeviceClientRecord>, ApiError> {
    if let Some(tenant_id) = tenant_id {
        if !is_platform_admin(effective) && !has_tenant(effective, tenant_id, "tenant.stats.read") {
            return Err(ApiError::Forbidden);
        }
    }
    if let Some(device_client_id) = device_client_id {
        let client = state.db.device_client(device_client_id).await?;
        if tenant_id.is_some() && client.tenant_id.as_deref() != tenant_id {
            return Err(ApiError::NotFound);
        }
        if !can_read_stats_for_client(principal, effective, &client) {
            return Err(ApiError::NotFound);
        }
        return Ok(vec![client]);
    }
    Ok(state
        .db
        .list_device_clients()
        .await?
        .into_iter()
        .filter(|client| tenant_id.is_none() || client.tenant_id.as_deref() == tenant_id)
        .filter(|client| can_read_stats_for_client(principal, effective, client))
        .collect())
}

fn filter_querylog_entries(
    upstream: &serde_json::Value,
    allowed_client_ids: &std::collections::HashSet<String>,
    since: OffsetDateTime,
    limit: usize,
) -> Vec<serde_json::Value> {
    upstream
        .get("data")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry
                .get("client_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|client_id| allowed_client_ids.contains(client_id))
        })
        .filter(|entry| {
            entry
                .get("time")
                .and_then(serde_json::Value::as_str)
                .and_then(|value| {
                    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
                        .ok()
                })
                .is_some_and(|timestamp| timestamp >= since)
        })
        .take(limit)
        .cloned()
        .collect()
}

fn aggregate_authorized_stats(
    upstream: &serde_json::Value,
    clients: &[DeviceClientRecord],
    since: OffsetDateTime,
) -> FilteredStatsResponse {
    let mut devices: Vec<DeviceStatsResponse> = clients
        .iter()
        .map(|client| DeviceStatsResponse {
            device_client_id: client.id.clone(),
            client_id: client.client_id.clone(),
            queries: 0,
            blocked: 0,
            cached: 0,
        })
        .collect();
    devices.sort_by(|left, right| left.device_client_id.cmp(&right.device_client_id));
    let entries = upstream.get("data").and_then(serde_json::Value::as_array);
    for entry in entries.into_iter().flatten() {
        let Some(client_id) = entry.get("client_id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(device) = devices
            .iter_mut()
            .find(|device| device.client_id == client_id)
        else {
            continue;
        };
        let in_window = entry
            .get("time")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| {
                OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
            })
            .is_some_and(|timestamp| timestamp >= since);
        if !in_window {
            continue;
        }
        device.queries += 1;
        if entry
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason.starts_with("Filtered"))
        {
            device.blocked += 1;
        }
        if entry
            .get("cached")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            device.cached += 1;
        }
    }
    let client_ids = devices
        .iter()
        .map(|device| device.client_id.clone())
        .collect();
    let queries = devices.iter().map(|device| device.queries).sum();
    let blocked = devices.iter().map(|device| device.blocked).sum();
    let cached = devices.iter().map(|device| device.cached).sum();
    FilteredStatsResponse {
        window_start: since
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
        client_ids,
        queries,
        blocked,
        cached,
        truncated: entries.is_some_and(|entries| entries.len() >= 5000),
        devices,
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

fn parse_rfc3339(value: &str) -> Result<OffsetDateTime, ApiError> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map_err(|_| ApiError::BadRequest)
}

fn format_rfc3339(value: OffsetDateTime) -> Result<String, ApiError> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| ApiError::Internal)
}

fn validate_batch_id(value: &str) -> Result<(), ApiError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
    {
        return Err(ApiError::BadRequest);
    }
    Ok(())
}

fn bearer_token(headers: &HeaderMap) -> &str {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .or_else(|| {
            headers
                .get("x-nabe-edge-token")
                .and_then(|value| value.to_str().ok())
        })
        .unwrap_or_default()
}

fn verify_enrollment_token(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = state
        .config
        .dev_edge_token
        .as_deref()
        .ok_or(ApiError::Forbidden)?;
    if constant_time_bytes_eq(bearer_token(headers).as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn verify_edge_credential(
    state: &AppState,
    headers: &HeaderMap,
    node_id: &str,
) -> Result<(), ApiError> {
    let token = bearer_token(headers);
    if !token.starts_with("nabe_edge_") || token.len() < 32 {
        return Err(ApiError::Forbidden);
    }
    state
        .db
        .authenticate_edge_node(node_id, &hash_edge_credential(token))
        .await
}

fn generate_edge_credential() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("nabe_edge_{}", URL_SAFE_NO_PAD.encode(bytes))
}

fn hash_edge_credential(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

fn constant_time_bytes_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
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
struct EdgeStatBucketRequest {
    client_id: String,
    bucket_start: String,
    bucket_seconds: i64,
    queries: u64,
    blocked: u64,
    cached: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EdgeStatsIngestRequest {
    batch_id: String,
    node_id: String,
    collected_through: String,
    buckets: Vec<EdgeStatBucketRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeStatsIngestResponse {
    accepted: bool,
    batch_id: String,
    bucket_count: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceClientCreateRequest {
    label: String,
    tenant_id: Option<String>,
    client_id: String,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceClientUpdateRequest {
    label: Option<String>,
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
    owner_provider: Option<String>,
    owner_subject: Option<String>,
    tenant_id: Option<String>,
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
            owner_provider: record.owner_provider,
            owner_subject: record.owner_subject,
            tenant_id: record.tenant_id,
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
    tenant_id: Option<String>,
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
    tenant_id: Option<String>,
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
            tenant_id: record.tenant_id,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    credential: Option<String>,
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
            credential: None,
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
        aggregate_authorized_stats, can_manage_device_client, can_read_device_client,
        compose_block_rules, filter_querylog_entries, validate_blocked_domains, validate_client_id,
        validate_domain, validate_label,
    };
    use crate::auth::{
        authorization::{EffectivePermissions, TenantPermissions},
        Principal,
    };
    use crate::db::repositories::{DeviceClientRecord, DnsPolicyRecord};
    use time::OffsetDateTime;

    fn policy(id: &str, domains: &str, enabled: bool) -> DnsPolicyRecord {
        DnsPolicyRecord {
            id: id.to_string(),
            name: format!("policy {id}"),
            blocked_domains_json: domains.to_string(),
            enabled,
            tenant_id: None,
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
        let rules = compose_block_rules(&policies, &[]).expect("rules");
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
        assert!(compose_block_rules(&policies, &[]).is_err());
    }

    #[test]
    fn tenant_rules_are_limited_to_enabled_tenant_clients() {
        let mut tenant_policy = policy("tenant", r#"["blocked.nabe.test"]"#, true);
        tenant_policy.tenant_id = Some("gray".to_string());
        let clients = vec![DeviceClientRecord {
            id: "device-1".to_string(),
            label: "Phone".to_string(),
            owner_scope: "tenant:gray".to_string(),
            owner_provider: Some("issuer.example".to_string()),
            owner_subject: Some("alice".to_string()),
            tenant_id: Some("gray".to_string()),
            client_id: "alice-phone".to_string(),
            enabled: true,
            created_at: "2026-07-11T00:00:00Z".to_string(),
            updated_at: "2026-07-11T00:00:00Z".to_string(),
        }];
        assert_eq!(
            compose_block_rules(&[tenant_policy], &clients).expect("rules"),
            vec!["||blocked.nabe.test^$client=nabe-alice-phone"]
        );
    }

    #[test]
    fn device_client_access_is_owner_or_exact_tenant_only() {
        let principal = Principal::identity("issuer.example", "alice", None, None);
        let client = DeviceClientRecord {
            id: "device-1".to_string(),
            label: "Phone".to_string(),
            owner_scope: "tenant:gray".to_string(),
            owner_provider: Some("issuer.example".to_string()),
            owner_subject: Some("bob".to_string()),
            tenant_id: Some("gray".to_string()),
            client_id: "bob-phone".to_string(),
            enabled: true,
            created_at: "2026-07-11T00:00:00Z".to_string(),
            updated_at: "2026-07-11T00:00:00Z".to_string(),
        };
        let tenant_permissions = |tenant_id: &str| EffectivePermissions {
            roles: vec!["baseline".to_string()],
            permissions: vec!["device_client.read_own".to_string()],
            tenants: vec![TenantPermissions {
                tenant_id: tenant_id.to_string(),
                tenant_slug: tenant_id.to_string(),
                roles: vec!["viewer".to_string()],
                permissions: vec!["tenant.device_client.read".to_string()],
            }],
        };
        assert!(can_read_device_client(
            &principal,
            &tenant_permissions("gray"),
            &client
        ));
        assert!(!can_read_device_client(
            &principal,
            &tenant_permissions("other"),
            &client
        ));
        assert!(!can_manage_device_client(
            &principal,
            &tenant_permissions("gray"),
            &client
        ));
    }

    #[test]
    fn querylog_filter_requires_exact_client_id_and_time_window() {
        let upstream = serde_json::json!({
            "oldest": "2026-07-12T09:00:00Z",
            "data": [
                {"client_id": "alice-phone", "time": "2026-07-12T11:30:00Z", "question": {"name": "allowed.example"}},
                {"client_id": "bob-phone", "time": "2026-07-12T11:31:00Z", "question": {"name": "private.example"}},
                {"client_id": "alice-phone", "time": "2026-07-12T09:00:00Z", "question": {"name": "old.example"}},
                {"client": "192.0.2.1", "time": "2026-07-12T11:32:00Z", "question": {"name": "unattributed.example"}}
            ]
        });
        let allowed = std::collections::HashSet::from(["alice-phone".to_string()]);
        let since = OffsetDateTime::parse(
            "2026-07-12T11:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .expect("since");
        let filtered = filter_querylog_entries(&upstream, &allowed, since, 10);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0]["client_id"], "alice-phone");
        assert_eq!(filtered[0]["question"]["name"], "allowed.example");
    }

    #[test]
    fn stats_aggregate_contains_no_domains_or_other_clients() {
        let upstream = serde_json::json!({
            "data": [
                {"client_id": "alice-phone", "time": "2026-07-12T11:30:00Z", "reason": "NotFilteredNotFound", "cached": true, "question": {"name": "allowed.example"}},
                {"client_id": "alice-phone", "time": "2026-07-12T11:31:00Z", "reason": "FilteredBlackList", "question": {"name": "blocked.example"}},
                {"client_id": "bob-phone", "time": "2026-07-12T11:32:00Z", "reason": "FilteredBlackList", "question": {"name": "private.example"}}
            ]
        });
        let client = DeviceClientRecord {
            id: "device-alice".to_string(),
            label: "Phone".to_string(),
            owner_scope: "subject:issuer.example:alice".to_string(),
            owner_provider: Some("issuer.example".to_string()),
            owner_subject: Some("alice".to_string()),
            tenant_id: None,
            client_id: "alice-phone".to_string(),
            enabled: true,
            created_at: "2026-07-11T00:00:00Z".to_string(),
            updated_at: "2026-07-11T00:00:00Z".to_string(),
        };
        let since = OffsetDateTime::parse(
            "2026-07-12T11:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .expect("since");
        let stats = aggregate_authorized_stats(&upstream, &[client], since);
        assert_eq!(stats.queries, 2);
        assert_eq!(stats.blocked, 1);
        assert_eq!(stats.cached, 1);
        assert_eq!(stats.devices.len(), 1);
        let encoded = serde_json::to_string(&stats).expect("serialize");
        assert!(!encoded.contains("allowed.example"));
        assert!(!encoded.contains("private.example"));
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
