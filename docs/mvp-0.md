# MVP 0

MVP 0 is the first running Nabe version for a self-contained Cloud DNS master stack.

## Scope

- SvelteKit Nabe Console with German-first UI and English as an option.
- Rust Axum Nabe API as the only backend.
- Turso Database Rust-rewrite embedded in the API container with file persistence at `/data/nabe.db`.
- Zitadel OIDC login handled by the backend using a public PKCE client.
- Every authenticated Zitadel user maps to the `admin` role.
- One AdGuard Home DNS Engine managed through the Nabe API.
- One local Unbound recursive resolver used by AdGuard Home.
- Dashboard shows backend session state and live AdGuard engine status.

## Non-Goals

- No PostgreSQL container.
- No Drizzle ORM.
- No TypeScript API backend.
- No Nabe user-management UI.
- No multi-user permissions beyond `authenticated = admin`.
- No Speiche deployment flow.
- No public native AdGuard UI or AdGuard API exposure by default.

## Data Ownership

The Nabe API owns all database access. The SvelteKit app never talks to Turso directly and never receives AdGuard credentials.

The API stores only the minimum local subject cache needed for the MVP:

- Zitadel provider and subject.
- Optional email and display name.
- Role mapping to `admin`.
- First and last seen timestamps.

## Local Runtime

The root `compose.dev.yaml` is the MVP runtime target. It runs:

- `nabe-web`
- `nabe-api`
- `dns-adguard`
- `dns-unbound`
- optional `dns-newt` profile for future break-glass access

The API persists the Turso database file through the `nabe-data` volume. AdGuard Home and Unbound remain in the same stack so the core DNS path can be tested without a separate external database service.

See [zitadel.md](zitadel.md) for the required Zitadel project, application, redirect URI, logout URI, token, and role settings.
