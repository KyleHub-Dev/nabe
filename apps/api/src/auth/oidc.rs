use axum::response::Redirect;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::{auth::Principal, config::Config, db::repositories::SubjectInput, error::ApiError};

#[derive(Debug, Deserialize)]
struct Discovery {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
}

pub async fn build_login_redirect(
    config: &Config,
    http: &Client,
) -> Result<(Redirect, Vec<String>), ApiError> {
    let client_id = config
        .oidc_client_id
        .as_deref()
        .ok_or(ApiError::OidcNotConfigured)?;
    let discovery = discover(config, http).await?;
    let state = random_urlsafe(32);
    let verifier = random_urlsafe(64);
    let challenge = pkce_challenge(&verifier);

    let mut url = Url::parse(&discovery.authorization_endpoint).map_err(|_| ApiError::Internal)?;
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", config.oidc_redirect_uri.as_str())
        .append_pair("response_type", "code")
        .append_pair("scope", "openid profile email")
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");

    let secure = config.oidc_redirect_uri.scheme() == "https";
    let cookies = super::session::create_oidc_cookies(&state, &verifier, secure);
    Ok((Redirect::to(url.as_str()), cookies))
}

pub async fn exchange_code_for_principal(
    config: &Config,
    http: &Client,
    code: &str,
    verifier: &str,
) -> Result<Principal, ApiError> {
    let client_id = config
        .oidc_client_id
        .as_deref()
        .ok_or(ApiError::OidcNotConfigured)?;
    let discovery = discover(config, http).await?;
    let token: TokenResponse = http
        .post(&discovery.token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("redirect_uri", config.oidc_redirect_uri.as_str()),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|_| ApiError::Internal)?
        .error_for_status()
        .map_err(|_| ApiError::Unauthorized)?
        .json()
        .await
        .map_err(|_| ApiError::BadUpstreamResponse)?;

    if let Some(userinfo_endpoint) = discovery.userinfo_endpoint {
        let userinfo: UserInfo = http
            .get(userinfo_endpoint)
            .bearer_auth(token.access_token)
            .send()
            .await
            .map_err(|_| ApiError::Internal)?
            .error_for_status()
            .map_err(|_| ApiError::Unauthorized)?
            .json()
            .await
            .map_err(|_| ApiError::BadUpstreamResponse)?;

        let display_name = userinfo.name.or(userinfo.preferred_username);
        return Ok(Principal::admin(
            "zitadel",
            userinfo.sub,
            userinfo.email,
            display_name,
        ));
    }

    Err(ApiError::BadUpstreamResponse)
}

pub fn subject_input(principal: &Principal) -> SubjectInput {
    SubjectInput {
        id: Uuid::new_v4().to_string(),
        provider: principal.provider.clone(),
        subject: principal.subject.clone(),
        email: principal.email.clone(),
        display_name: principal.display_name.clone(),
    }
}

async fn discover(config: &Config, http: &Client) -> Result<Discovery, ApiError> {
    let url = config
        .oidc_issuer_url
        .join(".well-known/openid-configuration")
        .map_err(|_| ApiError::Internal)?;
    http.get(url)
        .send()
        .await
        .map_err(|_| ApiError::Internal)?
        .error_for_status()
        .map_err(|_| ApiError::OidcNotConfigured)?
        .json()
        .await
        .map_err(|_| ApiError::BadUpstreamResponse)
}

fn random_urlsafe(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}
