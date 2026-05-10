use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("authentication required")]
    Unauthorized,
    #[error("OIDC is not configured")]
    OidcNotConfigured,
    #[error("invalid auth state")]
    InvalidAuthState,
    #[error("DNS Engine authentication failed")]
    AdguardAuthFailed,
    #[error("DNS Engine is not configured")]
    AdguardNotConfigured,
    #[error("DNS Engine is unreachable")]
    AdguardUnreachable,
    #[error("bad upstream response")]
    BadUpstreamResponse,
    #[error("database error")]
    Database,
    #[error("internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::OidcNotConfigured => "oidc_not_configured",
            Self::InvalidAuthState => "invalid_auth_state",
            Self::AdguardAuthFailed => "adguard_auth_failed",
            Self::AdguardNotConfigured => "adguard_not_configured",
            Self::AdguardUnreachable => "adguard_unreachable",
            Self::BadUpstreamResponse => "bad_upstream_response",
            Self::Database => "database_error",
            Self::Internal => "internal_error",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized | Self::InvalidAuthState => StatusCode::UNAUTHORIZED,
            Self::OidcNotConfigured | Self::AdguardNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            Self::AdguardAuthFailed => StatusCode::BAD_GATEWAY,
            Self::AdguardUnreachable | Self::BadUpstreamResponse => StatusCode::BAD_GATEWAY,
            Self::Database | Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            error: ErrorDetails {
                code: self.code(),
                message: self.to_string(),
            },
        };
        (status, Json(body)).into_response()
    }
}
