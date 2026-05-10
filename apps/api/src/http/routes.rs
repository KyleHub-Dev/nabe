use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    adguard::AdguardError,
    auth::{self, session, Principal},
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
    let adguard = match state.adguard.get_status().await {
        Ok(_) => "ok",
        Err(AdguardError::NotConfigured) => "not_configured",
        Err(AdguardError::AuthFailed) => "auth_failed",
        Err(AdguardError::Unreachable) => "error",
        Err(AdguardError::BadResponse) => "degraded",
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
    let engine = load_engine_status(&state).await?;
    let stats = match state.adguard.get_stats().await {
        Ok(stats) => DashboardStats {
            available: true,
            dns_queries: stats.dns_queries.unwrap_or_default(),
            blocked_filtering: stats.blocked_filtering.unwrap_or_default(),
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
        .get_raw("status")
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
        .get_raw("stats")
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
        .get_raw("querylog")
        .await
        .map(Json)
        .map_err(map_adguard_error)
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardStats {
    available: bool,
    dns_queries: u64,
    blocked_filtering: u64,
}

async fn load_engine_status(state: &AppState) -> Result<EngineStatus, ApiError> {
    let status = state
        .adguard
        .get_status()
        .await
        .map_err(map_adguard_error)?;
    let querylog = state.adguard.get_querylog_info().await.ok();
    let stats = state.adguard.get_stats_info().await.ok();

    Ok(EngineStatus {
        kind: "adguard-home",
        name: "Cloud DNS",
        connected: true,
        version: status.version,
        protection_enabled: status.protection_enabled.unwrap_or(false),
        query_log_enabled: querylog.and_then(|value| value.enabled).unwrap_or(false),
        statistics_enabled: stats.and_then(|value| value.enabled).unwrap_or(false),
        last_checked_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    })
}

fn map_adguard_error(error: AdguardError) -> ApiError {
    match error {
        AdguardError::NotConfigured => ApiError::AdguardNotConfigured,
        AdguardError::AuthFailed => ApiError::AdguardAuthFailed,
        AdguardError::Unreachable => ApiError::AdguardUnreachable,
        AdguardError::BadResponse => ApiError::BadUpstreamResponse,
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
