# ADR 0004: Edge Connectivity And Live Query Logs

Status: Accepted

Date: 2026-05-21

Supersedes: any earlier assumption that interactive edge log access should use
polling jobs.

## Context

Nabe should feel like one dashboard. When a user clicks "show DNS queries", the
experience should be close to opening the native AdGuard query panel.

At the same time, central should not store every raw DNS query from every edge
by default. Edges may be behind NAT, on residential networks, or in constrained
operational zones. Inbound edge APIs should not be required.

## Decision

Speiche maintains an outbound authenticated session to Nabe while the edge is
online.

Interactive reads use the live session:

```text
Browser -> Nabe API -> authorization check -> active Speiche session
        -> local AdGuard API -> filtered response stream -> Nabe -> Browser
```

Polling jobs are not used for interactive query-log browsing.

Durable background jobs remain the method for:

- Policy apply.
- Device/client config apply.
- Upgrades.
- Service restarts.
- Scheduled rollup collection.
- Retry-safe operational changes.

If an edge is offline, Nabe shows:

- Edge offline/degraded state.
- Last seen timestamp.
- Last synced health/inventory/stats.
- Cached recent raw logs only if that tenant/edge explicitly enabled a central
  raw-log cache.

## Consequences

The one-dashboard UX stays fast when the edge is online.

The dashboard remains honest when the edge is offline. Central cannot show raw
logs it never received.

Idle edge traffic should be small: keepalive, heartbeat, and occasional control
messages. Nabe does not keep a direct live connection to each AdGuard API; it
keeps one Speiche control session per edge.

## Security Considerations

Live log requests must be scoped capabilities, not arbitrary API forwarding.

A live log capability should include:

- Edge id.
- Tenant id.
- Allowed Device Client ids or engine client identifiers.
- Time window.
- Row limit.
- Request id.
- Expiration.

Speiche must reject requests outside this capability. Nabe must re-filter
returned rows before showing them.

## Implementation Notes

The live transport can be WebSocket, HTTP/2 streaming, or another authenticated
bidirectional channel. The transport choice is less important than the protocol
constraints and audit behavior.
