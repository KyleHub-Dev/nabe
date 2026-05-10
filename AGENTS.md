# Agent Instructions

These instructions apply to the whole repository.

## Product Boundaries

- Nabe is the product: a self-hosted DNS control plane.
- Nabe is not a DNS resolver and not an AdGuard fork.
- AdGuard Home is the first DNS Engine adapter. Treat native AdGuard UI as private break-glass/debug access only.
- Speiche is a generic Edge Node agent. It must not become AdGuard-specific in naming or architecture.
- The browser must never receive DNS Engine admin credentials.
- User-facing logs and stats must be filtered server-side by owned Device Client IDs.

## Stack

- Web: SvelteKit + TypeScript.
- API: Fastify + TypeScript.
- Worker: TypeScript.
- Database: PostgreSQL + Drizzle ORM.
- Validation: Zod through `@nabe/validation`.
- Auth: Zitadel via OIDC through `@nabe/auth`.
- Edge Agent: Go.
- CLI: Go.
- Package manager: pnpm workspaces.
- Runtime: Node.js LTS.
- Deployment: Docker/Podman-compatible Compose.

## Repository Layout

- `apps/web`: Nabe Console, AGPL-3.0-or-later.
- `apps/api`: Nabe API, AGPL-3.0-or-later.
- `apps/worker`: background worker, AGPL-3.0-or-later.
- `apps/speiche`: Speiche Agent, Apache-2.0.
- `apps/cli`: Nabe CLI, Apache-2.0.
- `packages/edge-protocol`: shared edge protocol, Apache-2.0.
- `packages/adguard-client`: AdGuard Home API client, Apache-2.0.
- `packages/db`: Drizzle schema and migrations, AGPL-3.0-or-later.
- `packages/auth`: Zitadel/OIDC helpers, AGPL-3.0-or-later.
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

## Security Rules

- Do not commit secrets, filled `.env` files, tokens, passwords, private keys, or credentials.
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

Run these before committing meaningful changes:

```sh
pnpm lint
pnpm build
pnpm test:go
scripts/verify-repo.sh
```

Use focused checks while iterating:

```sh
pnpm --filter @nabe/web lint
pnpm --filter @nabe/api lint
pnpm --filter @nabe/db lint
cd apps/speiche && go test ./...
cd apps/cli && go test ./...
```

## Implementation Guidance

- Prefer existing package boundaries over adding new packages.
- Put shared request/response validation in `packages/validation`.
- Keep DNS Engine integrations behind adapter/client boundaries.
- Keep Zitadel/OIDC-specific logic in `packages/auth` and API integration code.
- Add database schema changes in `packages/db`.
- Keep Speiche outbound-only by default.
- Keep CLI workflows operator-focused and avoid duplicating web product flows unless they are useful for automation.
