# Architecture

Status: Background.

Canonical decisions now live in `docs/adr/`, especially:

- `docs/adr/0001-architecture-decision-method.md`
- `docs/adr/0002-control-plane-zone-model.md`
- `docs/adr/0004-edge-connectivity-live-logs.md`

If this document conflicts with an accepted ADR, the ADR supersedes it.

Nabe is the product and control plane. DNS Engines do the resolver work.

## Components

- Nabe Web: browser UI for the Nabe Console.
- Nabe API: Rust backend authority for OIDC sessions, Turso persistence, DNS Engine adapters, and filtered query visibility.
- Nabe Worker: future background jobs for sync, cleanup, and edge processing. It is outside MVP 0 deployment.
- Nabe CLI: Go command-line tool for future admin/operator workflows.
- DB: Turso Database Rust rewrite, embedded in the Rust API container for MVP 0.
- AdGuard Home engine adapter: first DNS Engine integration.
- Unbound: local recursive resolver component per deployment.
- Speiche Agent: outbound edge connector for future Edge Nodes.
- Edge Nodes: future remote/local DNS nodes managed through Speiche.

## Data Flow

```text
User -> Nabe Web -> Nabe API -> AdGuard API
AdGuard Home -> Unbound
Speiche -> Nabe API -> jobs/heartbeat/status
```

The browser talks only to Nabe API. It never receives AdGuard admin credentials. Query logs and stats are filtered by the Nabe API before being returned to a user.

Native AdGuard UI is not the product UI. It is reserved for private break-glass/debug access because it cannot enforce Nabe's ownership model, UI language, audit rules, or server-side log filtering.

## Stack

- Web: SvelteKit + TypeScript.
- API: Rust + Axum.
- Worker: out of MVP 0 deployment.
- Database: Turso Database Rust rewrite.
- Validation: Zod, shared through `@nabe/validation`.
- Auth: Zitadel via OIDC handled by the Rust API.
- Authorization: Nabe-owned global roles, tenant group mappings, tenant roles, memberships, and fine-grained permissions.
- Edge Agent: Go.
- CLI: Go.
- Package manager: pnpm workspaces.
- Runtime: Node.js LTS.
- Deployment: Docker/Podman-compatible Compose.
