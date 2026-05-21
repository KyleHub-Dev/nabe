# ADR 0003: Authorization And Permission Model

Status: Accepted

Date: 2026-05-21

Supersedes: previous informal guidance in `docs/authorization.md` where it
conflicts with this ADR. `docs/authorization.md` remains background material.

## Context

Nabe needs platform administrators, group or family administrators, regular
users, viewers, and possibly restricted users. These roles must not leak across
tenants or edges. Query logs are especially sensitive because DNS history can
reveal browsing behavior, internal services, habits, and operational assets.

Zitadel can authenticate users and provide claims, but product authorization
must remain under Nabe's control.

## Decision

Nabe uses Nabe-owned authorization.

Zitadel proves identity. Nabe decides access.

Authorization decisions combine:

- Verified subject identity.
- Global role.
- Tenant membership.
- Tenant role.
- Resource ownership.
- Action-specific permission.
- Constraints such as time window, reason, edge state, or break-glass mode.

Roles grant capabilities, but resource scope decides which rows and logs are
visible.

Default roles:

| Role | Scope | Purpose |
| --- | --- | --- |
| `platform_admin` | Whole installation | Manage installation, tenants, engines, edges, secrets |
| `platform_operator` | Whole installation | Operate health, jobs, backups, updates without tenant log access |
| `tenant_owner` | One tenant | Own group/family/team tenant and delegate administration |
| `tenant_admin` | One tenant | Manage tenant devices, policies, and permitted log views |
| `tenant_viewer` | One tenant | Read tenant config and aggregated stats |
| `user` | Self plus assigned tenant | Manage own Device Clients and view own status/logs when allowed |
| `restricted_user` | Self | Use DNS with minimal self-service |

Permission namespaces:

- `platform.*`
- `tenant.*`
- `edge.*`
- `device.*`
- `policy.*`
- `stats.*`
- `querylog.*`
- `audit.*`

Every API route must be deny-by-default and declare the action and resource it
protects.

## Consequences

Tenant admins can manage their group without becoming platform operators.
Platform operators can troubleshoot infrastructure without automatically
reading tenant DNS history.

Per-user raw log views are allowed only when Nabe has reliable client identity.
If all traffic appears as one router IP, per-user log views must be disabled or
shown only at the tenant/network level.

## Security Considerations

Raw DNS log access must be:

- Permission checked.
- Tenant scoped.
- Device Client scoped when possible.
- Time-windowed.
- Audited.
- Re-filtered by Nabe even if the DNS engine also filters.

Break-glass access must require a reason and produce an audit event.

## Implementation Notes

The API should centralize authorization in middleware or handler guards. Tests
must include positive and negative cases for cross-tenant denial, owner-only
visibility, tenant admin limits, and platform operator limits.
