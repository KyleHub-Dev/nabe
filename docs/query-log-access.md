# Query log access

Raw DNS query logs remain in the DNS engine by default. Nabe fetches a bounded
live window only when an authorized user requests it and does not persist the
raw entries centrally.

`GET /api/engine/adguard/querylog` accepts:

- `reason`: required audit reason, 1 to 200 characters
- `tenantId`: optional exact tenant scope
- `deviceClientId`: optional exact Nabe Device Client record ID
- `since`: optional RFC 3339 timestamp, defaults to one hour ago and cannot be
  more than 24 hours ago
- `limit`: optional result limit from 1 to 500, default 100

The API computes the allowed ClientID set from current Nabe grants. Own access
requires `querylog.read_own`; tenant access requires
`tenant.querylog.read` for that exact tenant. Platform admins may inspect all
Nabe-owned clients. A requested Device Client outside the caller's scope is
reported as not found.

Nabe then filters every engine entry by exact `client_id` and timestamp before
returning it. Entries attributed only by source IP, entries without a ClientID,
and entries belonging to other Nabe clients are discarded. This fail-closed
behavior is required because source IP alone is not a reliable user identity.

Each successful view creates a durable audit event containing the actor,
tenant and Device Client scope, time window, allowed ClientIDs, result count,
and stated reason. Audit metadata never includes queried domains or response
payloads.

## Scoped statistics

`GET /api/engine/adguard/stats` accepts the same scope, `since`, and audit
`reason` parameters. It returns counts for queries, blocked responses, and
cache hits, plus per-Device-Client counts. It derives those counts only from
entries carrying an authorized exact ClientID. Domain names and raw response
data are never included in the statistics response or its audit event.

Own statistics require `stats.read_own`; tenant statistics require
`tenant.stats.read`. The response contains `truncated: true` if the bounded
engine read reached 5,000 entries, so callers are never led to treat a partial
live rollup as complete historical data. Durable long-term dashboards consume
the central five-minute buckets described in
[statistics-aggregation.md](statistics-aggregation.md).
