use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::{Duration, OffsetDateTime};

use crate::{auth::Principal, error::ApiError};

type HmacSha256 = Hmac<Sha256>;

const SESSION_COOKIE: &str = "nabe_session";
const OIDC_STATE_COOKIE: &str = "nabe_oidc_state";
const OIDC_VERIFIER_COOKIE: &str = "nabe_oidc_verifier";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionClaims {
    provider: String,
    subject: String,
    email: Option<String>,
    display_name: Option<String>,
    exp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcTransient {
    pub state: String,
    pub verifier: String,
}

pub fn read_principal(headers: &HeaderMap, secret: &str) -> Result<Principal, ApiError> {
    let cookie = get_cookie(headers, SESSION_COOKIE).ok_or(ApiError::Unauthorized)?;
    let claims: SessionClaims = verify_signed_json(&cookie, secret)?;

    if claims.exp < OffsetDateTime::now_utc().unix_timestamp() {
        return Err(ApiError::Unauthorized);
    }

    Ok(Principal::admin(
        claims.provider,
        claims.subject,
        claims.email,
        claims.display_name,
    ))
}

pub fn create_session_cookie(
    principal: &Principal,
    secret: &str,
    secure: bool,
) -> Result<String, ApiError> {
    let claims = SessionClaims {
        provider: principal.provider.clone(),
        subject: principal.subject.clone(),
        email: principal.email.clone(),
        display_name: principal.display_name.clone(),
        exp: (OffsetDateTime::now_utc() + Duration::days(7)).unix_timestamp(),
    };
    let value = sign_json(&claims, secret)?;
    Ok(format!(
        "{SESSION_COOKIE}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800{}",
        if secure { "; Secure" } else { "" }
    ))
}

pub fn clear_session_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub fn create_oidc_cookies(state: &str, verifier: &str, secure: bool) -> Vec<String> {
    let secure = if secure { "; Secure" } else { "" };
    vec![
        format!("{OIDC_STATE_COOKIE}={state}; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=600{secure}"),
        format!("{OIDC_VERIFIER_COOKIE}={verifier}; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=600{secure}"),
    ]
}

pub fn read_oidc_transient(headers: &HeaderMap) -> Result<OidcTransient, ApiError> {
    Ok(OidcTransient {
        state: get_cookie(headers, OIDC_STATE_COOKIE).ok_or(ApiError::InvalidAuthState)?,
        verifier: get_cookie(headers, OIDC_VERIFIER_COOKIE).ok_or(ApiError::InvalidAuthState)?,
    })
}

pub fn clear_oidc_cookies() -> Vec<String> {
    vec![
        format!("{OIDC_STATE_COOKIE}=; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=0"),
        format!("{OIDC_VERIFIER_COOKIE}=; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=0"),
    ]
}

fn sign_json<T: Serialize>(value: &T, secret: &str) -> Result<String, ApiError> {
    let payload = serde_json::to_vec(value).map_err(|_| ApiError::Internal)?;
    let payload = URL_SAFE_NO_PAD.encode(payload);
    let signature = sign(payload.as_bytes(), secret)?;
    Ok(format!("{payload}.{signature}"))
}

fn verify_signed_json<T: for<'de> Deserialize<'de>>(
    value: &str,
    secret: &str,
) -> Result<T, ApiError> {
    let (payload, signature) = value.split_once('.').ok_or(ApiError::Unauthorized)?;
    let expected = sign(payload.as_bytes(), secret)?;
    if !constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
        return Err(ApiError::Unauthorized);
    }

    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| ApiError::Unauthorized)?;
    serde_json::from_slice(&bytes).map_err(|_| ApiError::Unauthorized)
}

fn sign(payload: &[u8], secret: &str) -> Result<String, ApiError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| ApiError::Internal)?;
    mac.update(payload);
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    for cookie in cookies.split(';') {
        let (key, value) = cookie.trim().split_once('=')?;
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use axum::http::{header, HeaderMap, HeaderValue};

    use crate::auth::Principal;

    use super::{create_session_cookie, read_principal};

    #[test]
    fn signed_session_cookie_round_trips() {
        let principal = Principal::admin(
            "zitadel",
            "sub-1",
            Some("kyle@example.test".to_string()),
            Some("Kyle".to_string()),
        );
        let cookie = create_session_cookie(&principal, "test-secret", false).expect("cookie");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie.split(';').next().unwrap()).unwrap(),
        );

        let parsed = read_principal(&headers, "test-secret").expect("principal");
        assert_eq!(parsed.subject, "sub-1");
        assert_eq!(parsed.roles, vec!["admin"]);
    }

    #[test]
    fn signed_session_cookie_rejects_wrong_secret() {
        let principal = Principal::admin("zitadel", "sub-1", None, None);
        let cookie = create_session_cookie(&principal, "test-secret", false).expect("cookie");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie.split(';').next().unwrap()).unwrap(),
        );

        assert!(read_principal(&headers, "other-secret").is_err());
    }
}
