# Zitadel Configuration

Nabe MVP 0 uses Zitadel as the OpenID Connect provider. The Rust API owns the OIDC callback and creates the Nabe session cookie. The SvelteKit app only redirects the browser to the backend auth routes.

## Project

Create a Zitadel project, for example `nabe`.

In the project settings, enable:

- Return user roles during authentication.
- Only authorized users can authenticate.
- Authentication is restricted to users from organizations that have been granted access to this project.

This keeps Nabe authentication scoped to users who are explicitly allowed to use the project.

## Application

Create an application in the project:

- Name: `nabe-web`
- Type: `Web`
- Auth method: `PKCE`

For local development, enable Development Mode so `http://localhost` redirect URIs are accepted.

The application can be created before the redirect URIs are final. After creation, copy the Client ID and set it in Nabe:

```env
OIDC_CLIENT_ID=372369717538586629
```

Do not configure or store a client secret for the MVP public PKCE client.

## Redirect URIs

For the default local Compose setup, use:

```text
http://localhost:8080/auth/callback
```

This must match:

```env
OIDC_REDIRECT_URI=http://localhost:8080/auth/callback
```

The callback belongs to the Rust API, not the SvelteKit frontend.

## Post Logout URIs

For the default local Compose setup, use:

```text
http://localhost:5173/
```

This must match:

```env
OIDC_POST_LOGOUT_REDIRECT_URI=http://localhost:5173/
```

## Token Settings

In the application token settings:

- Set AuthTokenType to `JWT`.
- Enable profile information inside the ID token.

Zitadel roles are not trusted as Nabe product permissions. They may still be
enabled for other applications in the Zitadel project, but Nabe uses the
verified issuer and subject only and resolves authorization from its database.

## Nabe Role Mapping

Zitadel controls who can authenticate, while Nabe stores product authorization
through global roles, tenant roles, memberships, and permissions. Session
cookies store identity but no authority; current grants are loaded for every
protected request.

`baseline` is not a Zitadel project role. It is Nabe's implicit status for a signed-in user who has no tenant membership yet.

For tenant access, use a separate tenant group key such as `tenant_gray` and combine it with a tenant role such as `manager`, `user`, or `viewer`.

Nabe stores a local subject cache:

- provider
- subject
- optional email
- optional display name
- first seen timestamp
- last seen timestamp

To bootstrap the initial operator, obtain the user's Zitadel `sub` claim and
configure the exact issuer-host and subject pair:

```env
NABE_BOOTSTRAP_ADMIN_SUBJECTS=auth.kylehub.dev:<subject>
```

Multiple operators are comma-separated. This setting contains identifiers, not
credentials, but should still be managed as deployment configuration. Once a
verified login matches, Nabe idempotently grants its native `admin` role.

## Required Nabe Environment

```env
OIDC_ISSUER_URL=https://auth.kylehub.dev
OIDC_CLIENT_ID=372369717538586629
OIDC_REDIRECT_URI=http://localhost:8080/auth/callback
OIDC_POST_LOGOUT_REDIRECT_URI=http://localhost:5173/
SESSION_SECRET=change-this-to-a-local-random-secret
NABE_BOOTSTRAP_ADMIN_SUBJECTS=auth.kylehub.dev:<subject>
```

Use a unique local `SESSION_SECRET`. Do not commit filled `.env` files or real secrets.

## Post-MVP Service Account Automation

After MVP 0, Nabe may use a Zitadel service account to manage the Zitadel-side project setup automatically. This is intentionally not required for the MVP.

The service account would let Nabe verify or create the `nabe` project, the `nabe-web` PKCE application, redirect URIs, post-logout URIs, and project roles. Product permissions should still live in the Nabe database so future auth providers can be supported through provider adapters instead of tying all authorization logic to Zitadel.
