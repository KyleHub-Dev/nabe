# Device Clients And DNS Policies

Central owns device-client and DNS-policy records for MVP 0. Both are managed
through Nabe API endpoints and persisted in the central database, so they
survive central restarts. Applying policy pushes the resulting rules to the
active DNS engine through the AdGuard adapter; the native AdGuard UI stays
break-glass only.

## Device Clients

A device client is a DNS client owned by an OIDC subject and optionally attached
to a tenant, identified toward AdGuard by its ClientID value. Records contain a
stable id, human label, authoritative owner provider and subject, optional
tenant ID, the ClientID, an enabled flag, and created/updated timestamps.

Endpoints are session-authenticated and row-scoped:

```text
GET  /api/device-clients           list authorized device clients
POST /api/device-clients           create {label, clientId, tenantId?, enabled?}
GET  /api/device-clients/{id}      inspect one device client
PUT  /api/device-clients/{id}      partial update {label?, clientId?, enabled?}
```

Baseline users can manage records they own through
`device_client.read_own` and `device_client.manage_own`. Tenant access requires
the exact tenant's `tenant.device_client.read` or
`tenant.device_client.manage` permission. Platform admins can access all rows.
Unauthorized item lookups return not found so they do not reveal cross-tenant
resource existence. Owner and tenant fields cannot be reassigned through an
update payload.

ClientIDs are normalized to lowercase and restricted to `a-z`, `0-9`, and
inner hyphens, matching AdGuard ClientID rules. ClientIDs must be unique.
Disabling a device client (`enabled: false`) removes it from the active
client set reported by policy application.

## DNS Policies

A policy is a named set of blocked domains. `blocked.nabe.test` is the
deterministic verification domain used by the everything-working goal.

```text
GET  /api/dns-policies             list authorized policies
POST /api/dns-policies             create {name, blockedDomains, tenantId?, enabled?}
GET  /api/dns-policies/{id}        inspect one policy
PUT  /api/dns-policies/{id}        partial update {name?, blockedDomains?, enabled?}
POST /api/dns-policies/apply       apply all enabled policies to the engine
```

Domains are normalized to lowercase, must contain a dot, and are validated
per DNS label rules. Policies must contain at least one domain.

Tenant policy reads require `tenant.policy.read`; mutations require
`tenant.policy.manage`. A policy without a tenant is installation-global and
can only be managed by a platform admin.

## Apply Semantics

`POST /api/dns-policies/apply`:

1. Reconciles enabled Nabe Device Clients into namespaced AdGuard persistent
   clients such as `nabe-phone-client`; disabled Nabe clients are removed.
2. Collects the blocked domains of enabled policies, deduplicated and sorted
   for deterministic output.
3. Composes installation-global rules as `||domain^` and tenant rules as
   `||domain^$client=nabe-<ClientID>` for each enabled client in that tenant.
4. Pushes the full rule set through the adapter (`filtering/set_rules`),
   replacing previously applied Nabe rules.
5. Records `lastAppliedAt` on each applied policy and returns the applied
   policy ids, the rules, and the ClientIDs of enabled device clients.

Disabled policies contribute no rules. Disabled device clients are excluded
from the active client list and engine reconciliation. Tenant policies never
produce an unscoped rule. Apply is idempotent: re-applying the same state
produces the same rule set and updates the same namespaced persistent clients.
