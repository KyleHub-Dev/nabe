# Device Clients And DNS Policies

Central owns device-client and DNS-policy records for MVP 0. Both are managed
through Nabe API endpoints and persisted in the central database, so they
survive central restarts. Applying policy pushes the resulting rules to the
active DNS engine through the AdGuard adapter; the native AdGuard UI stays
break-glass only.

## Device Clients

A device client is a DNS client owned by a subject, identified toward AdGuard
by its ClientID value. Records contain a stable id, human label, an owner
scope placeholder for future tenants, the ClientID, an enabled flag, and
created/updated timestamps.

Endpoints (session-authenticated; reads require `edge.read`, writes require
`edge.manage`):

```text
GET  /api/device-clients           list all device clients
POST /api/device-clients           create {label, clientId, ownerScope?, enabled?}
GET  /api/device-clients/{id}      inspect one device client
PUT  /api/device-clients/{id}      partial update {label?, clientId?, ownerScope?, enabled?}
```

ClientIDs are normalized to lowercase and restricted to `a-z`, `0-9`, and
inner hyphens, matching AdGuard ClientID rules. ClientIDs must be unique.
Disabling a device client (`enabled: false`) removes it from the active
client set reported by policy application.

## DNS Policies

A policy is a named set of blocked domains. `blocked.nabe.test` is the
deterministic verification domain used by the everything-working goal.

```text
GET  /api/dns-policies             list all policies
POST /api/dns-policies             create {name, blockedDomains, enabled?}
GET  /api/dns-policies/{id}        inspect one policy
PUT  /api/dns-policies/{id}        partial update {name?, blockedDomains?, enabled?}
POST /api/dns-policies/apply       apply all enabled policies to the engine
```

Domains are normalized to lowercase, must contain a dot, and are validated
per DNS label rules. Policies must contain at least one domain.

## Apply Semantics

`POST /api/dns-policies/apply`:

1. Collects the blocked domains of all enabled policies, deduplicated and
   sorted for deterministic output.
2. Composes AdGuard user rules of the form `||domain^`.
3. Pushes the full rule set through the adapter (`filtering/set_rules`),
   replacing previously applied Nabe rules.
4. Records `lastAppliedAt` on each applied policy and returns the applied
   policy ids, the rules, and the ClientIDs of enabled device clients.

Disabled policies contribute no rules. Disabled device clients are excluded
from the active client list. Apply is idempotent: re-applying the same state
produces the same rule set.
