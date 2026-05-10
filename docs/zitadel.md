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
- Enable adding user roles to the access token.
- Enable user roles inside the ID token.
- Enable profile information inside the ID token.

The project setting for returning user roles during authentication must also remain enabled.

## Nabe Role Mapping

MVP 0 maps every successful Zitadel login to the Nabe `admin` role. Nabe does not manage users locally in this phase. Zitadel remains the source of truth for who can authenticate.

Nabe stores only a local subject cache:

- provider
- subject
- optional email
- optional display name
- role
- first seen timestamp
- last seen timestamp

## Required Nabe Environment

```env
OIDC_ISSUER_URL=https://auth.kylehub.dev
OIDC_CLIENT_ID=372369717538586629
OIDC_REDIRECT_URI=http://localhost:8080/auth/callback
OIDC_POST_LOGOUT_REDIRECT_URI=http://localhost:5173/
SESSION_SECRET=change-this-to-a-local-random-secret
```

Use a unique local `SESSION_SECRET`. Do not commit filled `.env` files or real secrets.
