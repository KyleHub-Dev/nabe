# Nabe

Nabe is a self-hosted DNS control plane by KyleHub.

Nabe is not a DNS resolver. It is the central hub for managing encrypted DNS clients, DNS engine instances, policies, ownership, auditability, and future edge nodes across personal, family, homelab, and small trusted networks.

Current status: early bootstrap. This repository contains the first monorepo structure, documentation, runnable placeholders, deployment skeletons, and license boundaries. It is not production-ready.

## Product Model

- Nabe Console: the primary web UI for users and admins.
- Nabe API: the backend that owns users, Device Clients, permissions, Client Tokens, audit logs, and DNS Engine access.
- DNS Engine: resolver backend managed through adapters. AdGuard Home is the first engine.
- Speiche Agent: a generic edge connector for future Edge Nodes.
- Cloud DNS: the initial single AdGuard Home backed master instance.

AdGuard Home is consumed through its API by the Nabe backend. Native AdGuard UI is break-glass/debug only and should not be publicly routed by default.

## Architecture Summary

The MVP 0 target is a single admin login through Zitadel, a Rust Nabe API with Turso Database persistence, and live status for one Cloud DNS master backed by AdGuard Home and Unbound.

The browser must never receive AdGuard admin credentials. DNS logs and stats must be filtered server-side by owned client IDs. Cloud DNS should only allow tokenized clients through AdGuard Allowed Clients / ClientIDs.

## Repository Layout

```text
apps/web          Nabe Console, AGPL-3.0-or-later
apps/api          Rust Nabe API, AGPL-3.0-or-later
apps/worker       background jobs placeholder, AGPL-3.0-or-later
apps/speiche      Speiche Agent, Apache-2.0
apps/cli          Go CLI for admin/operator workflows, Apache-2.0
packages/*        shared packages with package-level licenses
docs/             product, architecture, security, and roadmap docs
scripts/          repo setup and verification helpers
LICENSES/         canonical license texts
```

## Quick Start

```sh
pnpm install
pnpm dev
```

Self-contained Compose development stack:

```sh
cp .env.example .env
podman compose -f compose.dev.yaml up -d --build
```

The current services are placeholders:

- Web: SvelteKit Nabe Console stub with German default UI, English language option, and system/light/dark theme support.
- API: Rust Axum service with Turso persistence, Zitadel OIDC callback, and AdGuard Engine status endpoints.
- Worker: TypeScript startup stub, not part of MVP 0 Compose deployment.
- Speiche: Go agent stub with outbound-only behavior by default.
- CLI: Go command stub for future admin/operator workflows.

## Git Remotes

Codeberg is the primary Git host. GitHub is a reach/community mirror.

`origin` fetches from Codeberg and pushes `main` to both Codeberg and GitHub through multiple push URLs. The `github` remote is a convenience remote.

See [docs/git-remotes-and-mirroring.md](docs/git-remotes-and-mirroring.md).

## MVP Development Stack

The first real Nabe version is developed against a self-contained Compose stack with Nabe, Turso Database persistence inside the Rust API, AdGuard Home, Unbound, and optional Newt. See [docs/mvp-0.md](docs/mvp-0.md), [docs/mvp-deployment.md](docs/mvp-deployment.md), and [compose.dev.yaml](compose.dev.yaml).

## Licensing

Nabe uses mixed licensing by package/component:

- Nabe platform components: AGPL-3.0-or-later.
- Speiche Agent: Apache-2.0.
- Shared protocol / SDK / engine client libraries: Apache-2.0 when they do not contain platform/product logic.

Package-level `LICENSE` files are authoritative. Apache-licensed components must not import AGPL platform code. AGPL platform code may depend on Apache components.

## Security Warning

Do not expose the native AdGuard UI or AdGuard API publicly. Keep AdGuard credentials server-side only. Do not commit secrets, tokens, passwords, or filled `.env` files.
