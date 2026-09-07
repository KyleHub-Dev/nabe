# Working on Nabe

Nabe is the DNS control plane. Speiche connects edge nodes outbound; DNS engines
resolve queries. AdGuard Home is the first engine, accessed through adapters.

## Read for the task

- For product scope, read [docs/product.md](docs/product.md) and
  [docs/roadmap.md](docs/roadmap.md). The household pilot is the next milestone;
  broader historical goals do not add requirements. Distinguish working-tree
  changes from releases.
- For household, appliance or device terminology, read [CONTEXT.md](CONTEXT.md).
- For architecture or security decisions, read the relevant accepted record in
  [docs/adr/README.md](docs/adr/README.md). ADRs record decisions; check code and
  tests to establish what is implemented.
- For authorization changes, read [docs/authorization.md](docs/authorization.md).
  For query visibility or retention, also read [docs/security.md](docs/security.md)
  and the relevant telemetry ADR.
- For deployment changes, read [docs/mvp-deployment.md](docs/mvp-deployment.md).
  Keep the root `compose.dev.yaml` self-contained. The worker is outside that
  stack. Edge installation belongs to the CLI and Speiche.
- For edge enrollment or installation, read
  [docs/edge-enrollment.md](docs/edge-enrollment.md) and [apps/cli/README.md](apps/cli/README.md).
- For license-boundary changes, read [docs/licensing.md](docs/licensing.md).
  Package-level `LICENSE` files are authoritative.
- For UI design, read `https://kylehub.dev/branding.txt`. Use role tokens and
  Iconify/Lucide icons. Record material departures near the decision as
  `departs from branding.txt section <number>: <reason>`.

## Boundaries

- Keep DNS-engine credentials in backend/agent code and local secret files.
  The browser must never receive engine admin credentials. Keep secrets out of
  logs, examples, generated fixtures and Git.
- Filter user-facing query logs and statistics server-side by owned Device Client
  IDs and current permissions. Authentication alone does not grant administration.
- Keep native AdGuard administration private by default. DNS defaults must limit
  access to intended clients; edge nodes connect outbound through Speiche.
- Apache-2.0 packages must not import AGPL platform code. Shared protocol/client
  libraries stay free of platform product logic. AGPL code may use Apache code.
- Keep engine integrations in adapter/client packages. Keep OIDC-specific logic
  in the Rust API auth module and schema changes in explicit SQL migrations.
- Documentation is English. The UI defaults to German, offers English, and keeps
  system/light/dark theme support. Prefer German unless the detected language is
  English.

## Hosting

GitHub is canonical. Follow [docs/git-hosting.md](docs/git-hosting.md)
when configuring a checkout. Store Git credentials in a local credential helper.

## Verification

Use focused checks while iterating:

```sh
pnpm --filter @nabe/web lint
pnpm --filter @nabe/web build
env -u APPIMAGE -u APPDIR cargo check --manifest-path apps/api/Cargo.toml
pnpm test:api
pnpm test:go
```

Before committing meaningful changes, run the full repository gate once:

```sh
pnpm lint
pnpm build
pnpm test
pnpm verify
```

Report failures caused by existing unfinished work separately. Preserve that
work and keep unrelated implementation changes outside the current task.
