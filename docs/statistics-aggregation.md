# DNS statistics aggregation

Nabe stores privacy-preserving DNS statistics as five-minute counters. Raw DNS
queries remain in AdGuard Home on the edge. Speiche reads the local query log,
uses only entries with an explicit ClientID, and sends counts for queries,
blocked responses, and cache hits. Domains, questions, answers, source IPs, and
raw query records are never included in the ingestion payload.

## Edge collection

After a successful heartbeat, Speiche fetches the local AdGuard query log in
bounded pages and collects entries newer than its durable telemetry cursor.
Counts are grouped by ClientID and UTC five-minute bucket. Collection fails
closed if pagination exceeds the safety bound instead of advancing the cursor
and silently dropping data.

Before upload, Speiche writes the complete pending batch to
`/var/lib/nabe/speiche/identity.json`. A restart or network failure retries that
same batch ID. The cursor advances only after central returns the matching
batch ID. This makes retries safe without storing raw DNS data in Speiche.

When the installer configures an authenticated AdGuard instance, the root-only
systemd service also reads `/etc/nabe/adguard-ui.env`. Credentials stay local
to Speiche and are never sent to central or the browser.

## Central ingestion

`POST /api/edge/stats` uses the outbound edge token and accepts at most 1,000
buckets per batch. Central validates:

- the edge node already exists;
- batch IDs and collection timestamps;
- five-minute UTC alignment;
- configured retention bounds;
- non-negative counters with `blocked <= queries` and `cached <= queries`.

ClientIDs not owned by Nabe are ignored. Known ClientIDs are resolved to their
internal Device Client record before storage. The batch marker, additive bucket
updates, and retention cleanup commit in one database transaction. Replaying a
batch returns `accepted: false` and does not increment counters twice.

Retention defaults to 30 days and is configured with
`NABE_STATS_RETENTION_DAYS`. Cleanup runs during ingestion and covers both
statistic buckets and old idempotency markers.

## Historical access

`GET /api/stats/history` supports `tenantId`, `deviceClientId`, `from`, `to`,
and a required audit `reason`. The default range is the previous 24 hours, and
requests cannot reach outside configured retention.

Own reads require `stats.read_own`; tenant reads require `tenant.stats.read` for
the exact tenant. Platform admins can inspect all Nabe Device Clients. Central
filters by authorized internal Device Client IDs before returning buckets, and
the read creates a durable audit event without domain data.
