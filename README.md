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

Zitadel setup is documented in [docs/zitadel.md](docs/zitadel.md). The Nabe authorization model is documented in [docs/authorization.md](docs/authorization.md).

## Edge Install Guide

The current edge target is a Raspberry Pi or similar Debian/Raspberry Pi OS host
running the local DNS stack:

- AdGuard Home on port `53` for LAN DNS.
- Unbound on `127.0.0.1:5335` as the upstream recursive resolver.
- Speiche as the outbound edge agent that enrolls with the Nabe API and sends
  heartbeats.

The generated resolver defaults keep DNSSEC validation strict. Unbound serves
recently expired validated cache entries during transient upstream failures,
prefetches active records, and only enables IPv6 recursion when the host has an
IPv6 default route. AdGuard Home uses Unbound as its sole primary resolver and
an encrypted Cloudflare resolver only when Unbound does not respond within five
seconds. DNS service is restricted to loopback and private IPv4 networks.
The standard AdGuard DNS filter is enabled, raw query logs stay local with a
seven-day rotation interval, and aggregate statistics retain 30 days.

Running `nabe install --edge` again is an in-place upgrade. It refreshes
Unbound, AdGuard Home, Speiche, and Nabe-managed configuration while preserving
the Speiche identity, AdGuard data directory, and existing generated
break-glass credentials for the same alias.

Use `--reuse-enrollment` during an upgrade to retain the root-owned Speiche
token. An optional `--remote` updates only the central API URL:

```sh
nabe install --edge --reuse-enrollment \
  --remote "http://nabe-central.local:8080" \
  --adguard-ui-alias adguard.home
```

The central Nabe API must be reachable from the edge device. For the current
LAN setup, use:

```sh
CENTRAL_API="http://10.0.0.230:8080"
```

Install the Nabe CLI on the edge:

```sh
curl -fsSL https://codeberg.org/KyleHub/nabe/raw/branch/main/install.sh | bash
nabe version
```

Install and enroll in one step when the central URL and token are ready:

```sh
nabe install --edge \
  --remote "$CENTRAL_API" \
  --token "<edge-token>" \
  --adguard-ui-alias adguard.home
```

Install first and enroll later when the central URL or token is not final:

```sh
nabe install --edge \
  --defer-enrollment \
  --adguard-ui-alias adguard.home

nabe configure edge \
  --remote "$CENTRAL_API" \
  --token "<edge-token>"
```

`--adguard-ui-alias adguard.home` is optional. When enabled, the installer
detects the edge LAN IP, binds the native AdGuard UI to that IP, creates a local
DNS rewrite for `adguard.home`, and stores generated break-glass credentials in:

```sh
sudo cat /etc/nabe/adguard-ui.env
```

Verify the edge after install:

```sh
systemctl is-active unbound
systemctl is-active AdGuardHome
systemctl is-active speiche
dig +short @127.0.0.1 -p 5335 example.com
dig +short @127.0.0.1 example.com
dig +short @127.0.0.1 blocked.nabe.test A
```

Useful edge files:

- `/etc/nabe/speiche.env`: central API URL, enrollment token, Speiche settings.
- `/etc/nabe/adguard-ui.env`: optional break-glass AdGuard UI URL and
  credentials.
- `/var/lib/nabe/speiche/identity.json`: stable edge identity.
- `/opt/AdGuardHome/AdGuardHome.yaml`: generated AdGuard Home config.

More detail is in [docs/edge-enrollment.md](docs/edge-enrollment.md).

## Licensing

Nabe uses mixed licensing by package/component:

- Nabe platform components: AGPL-3.0-or-later.
- Speiche Agent: Apache-2.0.
- Shared protocol / SDK / engine client libraries: Apache-2.0 when they do not contain platform/product logic.

Package-level `LICENSE` files are authoritative. Apache-licensed components must not import AGPL platform code. AGPL platform code may depend on Apache components.

## Security Warning

Do not expose the native AdGuard UI or AdGuard API publicly. Keep AdGuard credentials server-side only. Do not commit secrets, tokens, passwords, or filled `.env` files.
