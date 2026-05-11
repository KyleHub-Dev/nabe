# Agent Instructions

These instructions apply to the whole repository. They guide future coding agents and human contributors.

## Product Boundaries

- Nabe is a KyleHub product.
- Nabe is the product: a central DNS control plane and web platform.
- Nabe is not a DNS resolver and not an AdGuard fork.
- Speiche is the generic Edge Agent and spoke connector for Edge Nodes.
- AdGuard Home is the first DNS Engine adapter, not the product identity. Treat native AdGuard UI as private break-glass/debug access only.
- Unbound is a local recursive resolver component per deployment when needed. It is infrastructure, not the product identity.
- The browser must never receive DNS Engine admin credentials.
- User-facing logs and stats must be filtered server-side by owned Device Client IDs.

## Stack

- Web: SvelteKit + TypeScript.
- API: Rust + Axum.
- Worker: out of MVP 0 deployment unless explicitly reintroduced.
- Database: Turso Database Rust rewrite, embedded in the Rust API container.
- Validation: Zod through `@nabe/validation`.
- Auth: Zitadel via OIDC handled by the Rust API.
- Edge Agent: Go.
- CLI: Go.
- Package manager: pnpm workspaces.
- Runtime: Node.js LTS.
- Deployment: root-level Docker/Podman-compatible `compose.dev.yaml` for the self-contained MVP stack.

## Repository Layout

- `apps/web`: Nabe Console, AGPL-3.0-or-later.
- `apps/api`: Rust Nabe API, AGPL-3.0-or-later.
- `apps/worker`: background worker, AGPL-3.0-or-later.
- `apps/speiche`: Speiche Agent, Apache-2.0.
- `apps/cli`: Nabe CLI, Apache-2.0.
- `packages/edge-protocol`: shared edge protocol, Apache-2.0.
- `packages/adguard-adapter`: AdGuard Home engine adapter, Apache-2.0.
- `packages/validation`: shared Zod schemas, AGPL-3.0-or-later.
- `packages/config`: shared config loading, Apache-2.0 unless it imports AGPL code.
- `packages/ui`: shared UI components, AGPL-3.0-or-later.

## Licensing Rules

- Package-level `LICENSE` files are authoritative.
- Apache-2.0 packages must not import AGPL-licensed platform code.
- AGPL-licensed platform code may depend on Apache-2.0 packages.
- Keep shared protocol/client libraries free of platform product logic if they are Apache-2.0.

## Language And UI Rules

- README and docs are in English.
- The app defaults to German UI text.
- English must remain available as a UI language.
- Browser/system language detection should prefer German unless English is detected.
- Keep system/light/dark theme support in the UI architecture.

## KyleHub Branding

Follow the KyleHub brand guidance from `https://kylehub.dev/branding.txt`.

- Default identity: quiet one-operator technical surface, not SaaS marketing.
- Prefer flat, square, hairlined surfaces. Avoid rounded SaaS panels, gradients, blobs, glows, and glassmorphism.
- Use role tokens such as `--accent`, `--accent-warm`, `--surface`, `--text`, and `--hairline` instead of literal brand colors in component CSS.
- Use Bloom/Bark/Stone/Lichen/Hairline roles as described by the brand brief when designing product surfaces.
- Prefer direct copy. Avoid sales-led language and em dashes.
- Prefer Iconify for icons, with Lucide as the generic UI root set.
- Document intentional departures near the decision using: `departs from branding.txt section <number>: <reason>`.

## Security Rules

- Do not commit secrets, filled `.env` files, tokens, passwords, private keys, or credentials.
- Do not expose secrets through logs, client bundles, screenshots, examples, generated fixtures, or documentation.
- Keep AdGuard credentials server-side only.
- Do not expose AdGuard UI or AdGuard API publicly by default.
- Do not create open-resolver defaults.
- Cloud DNS should use encrypted DNS with ClientID tokens.
- Edge Nodes should connect outbound through Speiche instead of requiring inbound ports.

## Git And Hosting

- Codeberg is primary: `ssh://git@codeberg.org/KyleHub/nabe.git`.
- GitHub is only a mirror: `https://github.com/KyleHub-Dev/nabe.git`.
- `origin` fetches from Codeberg and pushes to both Codeberg and GitHub.
- Keep GitHub mirror-specific workflows out of `.github/` while GitHub Actions are disabled.
- Do not store GitHub credentials in the repository.

## Development Checks

Run the full repository gate before committing meaningful changes:

```sh
pnpm lint
pnpm build
pnpm test
pnpm verify
```

Use focused checks while iterating, based on the files changed:

```sh
# Web app or TypeScript workspace packages:
pnpm --filter @nabe/web lint
pnpm --filter @nabe/web build
pnpm --filter <package-name> lint

# Rust API:
env -u APPIMAGE -u APPDIR cargo check --manifest-path apps/api/Cargo.toml
pnpm test:api

# Go edge agent or CLI:
cd apps/speiche && go test ./...
cd apps/cli && go test ./...
```

Prefer the narrowest focused check that covers the change during development, then run the full gate once the change is ready. If a focused check fails because of unrelated existing work, note that clearly instead of widening the change.

## Implementation Guidance

- Prefer small, composable, boring infrastructure.
- Do not overbuild beyond the current product phase.
- Prefer existing package boundaries over adding new packages.
- Put shared request/response validation in `packages/validation`.
- Keep engine adapter code in permissive packages such as `packages/adguard-adapter`; do not duplicate AdGuard client logic inside the AGPL API.
- Keep DNS Engine integrations behind adapter/client boundaries.
- For MVP deployment work, keep one root-level self-contained `compose.dev.yaml` plus `.env.example`.
- Do not add separate `deploy/` directories for Nabe or Speiche during MVP 0.
- Speiche deployment is not MVP 0. Later, Speiche should be installable through Nabe CLI/control tooling.
- Keep Zitadel/OIDC-specific logic in the Rust API `auth` module.
- Add database schema changes as explicit SQL migrations in `apps/api/migrations`.
- Keep Speiche outbound-only by default.
- Keep CLI workflows operator-focused and avoid duplicating web product flows unless they are useful for automation.
