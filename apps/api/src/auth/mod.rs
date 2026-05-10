pub mod oidc;
pub mod session;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Principal {
    pub provider: String,
    pub subject: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

impl Principal {
    pub fn admin(
        provider: impl Into<String>,
        subject: impl Into<String>,
        email: Option<String>,
        display_name: Option<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            subject: subject.into(),
            email,
            display_name,
            roles: vec!["admin".to_string()],
        }
    }
}
