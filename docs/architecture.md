# Architecture

Nabe is the product and control plane. DNS Engines do the resolver work.

## Components

- Nabe Web: browser UI for the Nabe Console.
- Nabe API: backend authority for users, Device Clients, permissions, Client Tokens, audit logs, DNS Engine adapters, and filtered query visibility.
- Nabe Worker: background jobs for sync, cleanup, and future edge processing.
- Nabe CLI: Go command-line tool for future admin/operator workflows.
- DB: PostgreSQL system of record.
- AdGuard Home engine adapter: first DNS Engine integration.
- Speiche Agent: outbound edge connector for future Edge Nodes.
- Edge Nodes: future remote/local DNS nodes managed through Speiche.

## Data Flow

```text
User -> Nabe Web -> Nabe API -> AdGuard API
Speiche -> Nabe API -> jobs/heartbeat/status
```

The browser talks only to Nabe API. It never receives AdGuard admin credentials. Query logs and stats are filtered by the Nabe API before being returned to a user.

Native AdGuard UI is not the product UI. It is reserved for private break-glass/debug access because it cannot enforce Nabe's ownership model, UI language, audit rules, or server-side log filtering.

## Stack

- Web: SvelteKit + TypeScript.
- API: Fastify + TypeScript.
- Worker: TypeScript.
- Database: PostgreSQL + Drizzle ORM.
- Validation: Zod, shared through `@nabe/validation`.
- Auth: Zitadel via OIDC, with helpers staged in `@nabe/auth`.
- Edge Agent: Go.
- CLI: Go.
- Package manager: pnpm workspaces.
- Runtime: Node.js LTS.
- Deployment: Docker/Podman-compatible Compose.
