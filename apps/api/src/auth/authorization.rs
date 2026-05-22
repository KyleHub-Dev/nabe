use serde::Serialize;

use crate::{auth::Principal, error::ApiError};

pub const PLATFORM_ADMIN: &str = "platform.admin";
pub const EDGE_READ: &str = "edge.read";
pub const EDGE_MANAGE: &str = "edge.manage";
pub const STATS_READ_TENANT: &str = "stats.read_tenant";
pub const QUERYLOG_READ_TENANT: &str = "querylog.read_tenant";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePermissions {
    pub roles: Vec<String>,
    pub permissions: Vec<&'static str>,
}

pub fn effective_permissions(principal: &Principal) -> EffectivePermissions {
    let mut permissions = Vec::new();
    for role in &principal.roles {
        match role.as_str() {
            "admin" | "platform_admin" => {
                permissions.extend([
                    PLATFORM_ADMIN,
                    EDGE_READ,
                    EDGE_MANAGE,
                    STATS_READ_TENANT,
                    QUERYLOG_READ_TENANT,
                ]);
            }
            "tenant_owner" | "tenant_admin" => {
                permissions.extend([EDGE_READ, STATS_READ_TENANT, QUERYLOG_READ_TENANT]);
            }
            "tenant_viewer" => {
                permissions.extend([EDGE_READ, STATS_READ_TENANT]);
            }
            "user" | "restricted_user" => {}
            _ => {}
        }
    }
    permissions.sort_unstable();
    permissions.dedup();
    EffectivePermissions {
        roles: principal.roles.clone(),
        permissions,
    }
}

pub fn require(principal: &Principal, permission: &'static str) -> Result<(), ApiError> {
    let effective = effective_permissions(principal);
    if effective.permissions.contains(&permission) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::Principal;

    use super::{effective_permissions, require, PLATFORM_ADMIN, QUERYLOG_READ_TENANT};

    #[test]
    fn admin_has_platform_permissions() {
        let principal = Principal::admin("zitadel", "sub", None, None);
        assert!(require(&principal, PLATFORM_ADMIN).is_ok());
    }

    #[test]
    fn user_does_not_have_querylog_permissions() {
        let mut principal = Principal::admin("zitadel", "sub", None, None);
        principal.roles = vec!["user".to_string()];
        assert!(require(&principal, QUERYLOG_READ_TENANT).is_err());
        assert!(!effective_permissions(&principal)
            .permissions
            .contains(&QUERYLOG_READ_TENANT));
    }
}
