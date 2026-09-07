# Nabe

Nabe is a self-hosted dashboard for household DNS appliances. The first target is
one Raspberry Pi per home, with guided setup and filtering managed through Nabe.
AdGuard Home and Unbound provide local DNS; Speiche connects the appliance to
central without inbound router ports.

The next milestone covers three households: Kyle's apartment, his parents' home
and a friend's family home. They install Pi OS, connect Ethernet and run the
Nabe installer before completing browser setup.
Read the [product scope](docs/product.md) and [pilot roadmap](docs/roadmap.md).
The browser wizard and local recovery page are planned, not shipped.

Current status: under development, not production-ready. This checkout includes
control-plane APIs, device/policy management and edge installation. The household
wizard, remote policy application and recovery experience remain
planned work; these docs describe the development source, not a released version.

## Product Model

- Nabe Console: the primary web UI for users and admins.
- Nabe API: the backend that owns users, Device Clients, permissions, Client Tokens, audit logs, and DNS Engine access.
- DNS Engine: resolver backend managed through adapters. AdGuard Home is the first engine.
- Speiche Agent: the outbound connection from a household appliance to Nabe.
- Household appliance: one Edge Node running AdGuard Home and Unbound.
- Cloud DNS: an existing development configuration, outside the household pilot.

AdGuard Home is consumed through its API by the Nabe backend. Native AdGuard UI is break-glass/debug only and should not be publicly routed by default.

## Architecture Summary

The central API supports generic OIDC authentication, Nabe-owned global and
tenant authorization in Turso, durable control-plane audit events, and live
status for one Cloud DNS master backed by AdGuard Home and Unbound. Zitadel is
the documented first OIDC provider.

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

The current components are:

- Web: SvelteKit Nabe Console stub with German default UI, English language option, and system/light/dark theme support.
- API: Rust Axum service with Turso persistence, Zitadel OIDC callback, and AdGuard Engine status endpoints.
- Worker: TypeScript startup stub, not part of MVP 0 Compose deployment.
- Speiche: outbound Go edge agent with enrollment, heartbeats and an AdGuard driver.
- CLI: Go tool for edge installation, upgrades and configuration.

## Git hosting

[GitHub](https://github.com/KyleHub-Dev/nabe) is the canonical repository.
`origin` fetches and pushes there. For existing checkouts, see the
[remote setup instructions](docs/git-hosting.md).

Anonymous CLI and Speiche downloads require public repository access. While the
repository is private, use an authorized checkout and the build instructions in
[apps/cli/README.md](apps/cli/README.md).

## MVP Development Stack

The central development environment is a self-contained Compose stack with Nabe, Turso persistence inside the Rust API, AdGuard Home, Unbound, and optional Newt. Its central DNS engine is a development target; the household pilot requires policy application to a separate appliance. See [docs/mvp-0.md](docs/mvp-0.md), [docs/mvp-deployment.md](docs/mvp-deployment.md), and [compose.dev.yaml](compose.dev.yaml).

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
CENTRAL_API="http://nabe-central.local:8080"
```

Install the Nabe CLI on the edge:

```sh
curl -fsSL https://raw.githubusercontent.com/KyleHub-Dev/nabe/main/install.sh | bash
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
