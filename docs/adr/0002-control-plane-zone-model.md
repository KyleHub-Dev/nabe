# ADR 0002: Nabe Control Plane And Zone Model

Status: Accepted

Date: 2026-05-21

Supersedes: previous informal guidance in `docs/architecture.md`,
`docs/security.md`, `docs/adguard-engine.md`, and `docs/speiche.md` where those
files conflict with this ADR.

## Context

Nabe manages central and edge DNS engines. The product goal is one coherent
dashboard for users and operators, but the system must not collapse every DNS
engine, permission boundary, and log source into one unbounded trust zone.

AdGuard Home is the first DNS engine. Speiche is the edge management connector.
Zitadel is the identity provider. Nabe API is the product and policy authority.

## Decision

Nabe is the central control plane. DNS engines remain data-plane components.

The system is divided into zones:

| Zone | Assets | Trust role | Allowed conduits |
| --- | --- | --- | --- |
| Identity zone | Zitadel, OIDC keys, MFA | Authenticates users | OIDC redirects and token validation |
| Central control zone | Nabe API, DB, web, central adapter | Policy decision, audit, desired state | Browser/API, OIDC, Speiche sessions, engine API |
| Edge management zone | Speiche, node identity, local config | Constrained executor | Outbound authenticated session to Nabe |
| DNS data zone | AdGuard Home, Unbound, raw query logs | Local DNS processing | Localhost API from Speiche, DNS from clients |
| Client network zone | Phones, laptops, routers, OT devices | Untrusted DNS clients | DNS only, preferably identified by ClientID or DHCP identity |

Nabe Web talks only to Nabe API. It never receives AdGuard credentials.

Native AdGuard UI is private break-glass/debug access. It is not product UX and
must not be required for normal operations.

## Consequences

Central can coordinate policy, edge state, devices, and permissions without
becoming the universal owner of every raw DNS event.

Speiche is a conduit between zones. It must be constrained, authenticated,
audited, and protocol-limited. It is not a generic remote shell.

Central failure must not stop an already-configured edge from serving DNS.

## Security Considerations

The most important risk is central-to-edge blast radius. A central compromise
must not automatically grant arbitrary execution on every edge.

The edge conduit must support revocation, command allowlists, typed operations,
and per-command audit.

## Implementation Notes

The system model is explained visually in `docs/system-model.html`.
