# ADR 0005: Telemetry, Retention, And Log Access

Status: Accepted

Date: 2026-05-21

Supersedes: previous informal statements that query logs are simply consumed by
Nabe and filtered server-side, where that could imply permanent central raw-log
storage.

## Context

Nabe needs observability, debugging, user-facing stats, audit trails, and query
visibility. DNS query logs are sensitive. A central database containing every
tenant's raw DNS history would be high value and high risk.

## Decision

Nabe separates telemetry into classes:

| Class | Default home | Central storage | Access |
| --- | --- | --- | --- |
| Heartbeat and inventory | Speiche -> central | Yes | `edge.read` |
| Aggregated DNS stats | Edge or engine rollup -> central | Yes, time buckets | `stats.read_*` |
| Raw DNS query logs | DNS engine local storage | No by default | `querylog.read_*`, audited |
| Control-plane audit logs | Central | Yes, append-oriented | `audit.read` scoped by role |
| Service logs | Local or log collector | Redacted structured events only | Operator scoped |

Central raw-log caching is optional and must be explicit per tenant or edge. If
enabled, it must be encrypted, short-retention, access-controlled, and audited.

Application logs use structured wide events for control-plane requests and jobs.
They must not include secrets, session cookies, token material, AdGuard
credentials, or full raw DNS query payloads.

## Consequences

Nabe can provide useful dashboards and historical stats without centralizing all
raw browsing history.

Offline edge raw-log availability is limited unless optional cache is enabled.
This is a deliberate privacy and blast-radius tradeoff.

## Security Considerations

Audit logs and raw query logs are different data classes:

- Audit logs say who changed or viewed what.
- DNS query logs say what clients tried to resolve.

Audit logs belong centrally. Raw DNS logs do not, by default.

All raw-log views must create audit events. Audit events should include subject,
tenant, action, resource, time window, edge, request id, decision, and reason
when applicable.

## Implementation Notes

Rollups should prefer counts, timing, policy ids, rule ids, edge ids, tenant ids,
and Device Client ids. Avoid central raw domain storage unless a tenant setting
enables it.
