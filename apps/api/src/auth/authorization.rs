use serde::Serialize;

use crate::{
    auth::Principal,
    db::repositories::{AuthorizationRecord, TenantAuthorization},
    error::ApiError,
};

pub const PLATFORM_ADMIN: &str = "platform.admin";
pub const EDGE_READ: &str = "edge.read";
pub const EDGE_MANAGE: &str = "edge.manage";
pub const AUDIT_READ: &str = "audit.read";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantPermissions {
    pub tenant_id: String,
    pub tenant_slug: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePermissions {
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub tenants: Vec<TenantPermissions>,
}

impl From<AuthorizationRecord> for EffectivePermissions {
    fn from(record: AuthorizationRecord) -> Self {
        Self {
            roles: record.global_roles,
            permissions: record.global_permissions,
            tenants: record
                .tenants
                .into_iter()
                .map(TenantPermissions::from)
                .collect(),
        }
    }
}

impl From<TenantAuthorization> for TenantPermissions {
    fn from(tenant: TenantAuthorization) -> Self {
        Self {
            tenant_id: tenant.tenant_id,
            tenant_slug: tenant.tenant_slug,
            roles: tenant.roles,
            permissions: tenant.permissions,
        }
    }
}

pub fn principal_with_roles(
    mut principal: Principal,
    effective: &EffectivePermissions,
) -> Principal {
    principal.roles = effective.roles.clone();
    principal
}

pub fn require_global(effective: &EffectivePermissions, permission: &str) -> Result<(), ApiError> {
    if effective
        .permissions
        .iter()
        .any(|candidate| candidate == permission)
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

pub fn require_tenant(
    effective: &EffectivePermissions,
    tenant_id: &str,
    permission: &str,
) -> Result<(), ApiError> {
    if effective.tenants.iter().any(|tenant| {
        tenant.tenant_id == tenant_id
            && tenant
                .permissions
                .iter()
                .any(|candidate| candidate == permission)
    }) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::{require_global, require_tenant, EffectivePermissions, TenantPermissions};

    fn permissions() -> EffectivePermissions {
        EffectivePermissions {
            roles: vec!["operator".to_string()],
            permissions: vec!["edge.read".to_string()],
            tenants: vec![TenantPermissions {
                tenant_id: "gray".to_string(),
                tenant_slug: "gray".to_string(),
                roles: vec!["viewer".to_string()],
                permissions: vec!["tenant.read".to_string()],
            }],
        }
    }

    #[test]
    fn global_permissions_are_explicit() {
        assert!(require_global(&permissions(), "edge.read").is_ok());
        assert!(require_global(&permissions(), "edge.manage").is_err());
    }

    #[test]
    fn tenant_permission_cannot_cross_tenants() {
        assert!(require_tenant(&permissions(), "gray", "tenant.read").is_ok());
        assert!(require_tenant(&permissions(), "other", "tenant.read").is_err());
    }
}
