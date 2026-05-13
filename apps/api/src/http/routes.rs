use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use nabe_adguard_adapter::AdguardAdapterError;

use crate::{
    auth::{self, session, Principal},
    db::repositories::{EdgeNodeInput, EdgeNodeRecord},
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
        .route("/api/dashboard", get(dashboard))
        .route("/api/engine/status", get(engine_status))
        .route("/api/engine/adguard/status", get(adguard_status))
        .route("/api/engine/adguard/stats", get(adguard_stats))
        .route("/api/engine/adguard/querylog", get(adguard_querylog))
        .route("/api/dev/edge-token", get(dev_edge_token))
        .route("/api/edge/enroll", post(edge_enroll))
        .route("/api/edge/heartbeat", post(edge_heartbeat))
        .route("/api/edge/nodes", get(edge_nodes))
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

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DashboardResponse>, ApiError> {
    let principal = session::read_principal(&headers, &state.config.session_secret)?;
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
    let _ = session::read_principal(&headers, &state.config.session_secret)?;
    Ok(Json(load_engine_status(&state).await?))
}

async fn adguard_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _ = session::read_principal(&headers, &state.config.session_secret)?;
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
    let _ = session::read_principal(&headers, &state.config.session_secret)?;
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
    let _ = session::read_principal(&headers, &state.config.session_secret)?;
    state
        .adguard
        .raw_get("querylog")
        .await
        .map(Json)
        .map_err(map_adguard_error)
}

async fn dev_edge_token(
    State(state): State<AppState>,
) -> Result<Json<DevEdgeTokenResponse>, ApiError> {
    Ok(Json(DevEdgeTokenResponse {
        configured: state.config.dev_edge_token.is_some(),
        token: state.config.dev_edge_token.clone(),
        warning: "dev-only bootstrap token; do not use for production enrollment",
    }))
}

async fn edge_nodes(State(state): State<AppState>) -> Result<Json<EdgeNodesResponse>, ApiError> {
    let config = state.config.clone();
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
