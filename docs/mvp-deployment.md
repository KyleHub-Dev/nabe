# MVP Deployment Target

The first real Nabe version should be developed against a single self-contained Compose stack that contains Nabe, Turso Database persistence inside the Rust API, AdGuard Home, Unbound, and optional Newt.

The repository ships this stack at the repository root:

```text
compose.dev.yaml
```

## MVP Assumption

Nabe runs beside the DNS components, not as a separate platform first:

- `nabe-web`: Nabe Console.
- `nabe-api`: Rust backend authority for OIDC sessions, Turso persistence, DNS Engine access, and filtered logs.
- `dns-adguard`: first DNS Engine.
- `dns-unbound`: local recursive resolver used by AdGuard Home.
- `dns-newt`: optional private debug/admin route through Pangolin.

The API talks to AdGuard Home on the internal Compose network:

```text
ADGUARD_BASE_URL=http://dns-adguard:3000
```

The browser talks to Nabe only. It must not receive AdGuard credentials.

## Public Exposure Defaults

- Do not expose native AdGuard UI publicly by default.
- Do not expose AdGuard API publicly by default.
- Do not expose public plain DNS/53 by default.
- DoT/DoH may be exposed intentionally for Cloud DNS.
- Newt/Pangolin may route private debug/admin access, but it is not the main control plane.

## Development Flow

```sh
cp .env.example .env
podman compose -f compose.dev.yaml up -d --build
```

Enable the Newt service only when credentials are configured:

```sh
podman compose -f compose.dev.yaml --profile edge up -d --build
```

Speiche deployment is intentionally out of scope for MVP 0. Speiche is expected to be installed later through Nabe CLI/control tooling that can install or manage an edge Compose stack.

## What To Build First

This deployment shape makes the first MVP concrete:

1. Nabe API can connect to AdGuard over the internal Compose network.
2. Nabe stores its MVP state in Turso Database at `/data/nabe.db`.
3. Nabe handles Zitadel OIDC login/callback in the Rust API.
4. Nabe Console shows authenticated user and live DNS Engine status.
5. Newt/Pangolin remains optional for private native UI debugging.
