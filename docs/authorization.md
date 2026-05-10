# Authorization Model

Nabe uses Zitadel for authentication and Nabe-owned authorization data for product permissions.

For MVP 0 the backend still treats a successful Zitadel login as an admin session so the first local stack remains easy to operate. The database schema is prepared for the next step: multi-tenant authorization with global permissions, tenant roles, and subject memberships.

## Concepts

- Subject: a signed-in identity from an external provider, currently Zitadel OIDC.
- Global role: a platform-wide role that is not bound to a tenant.
- Tenant: an isolated customer, family, team, or organization scope.
- Tenant role: a role granted to a subject inside one tenant.
- Permission: a fine-grained action Nabe can check before returning data or changing state.
- Device Client: a DNS client owned by a subject and optionally attached to a tenant.

## Global Roles

`admin` is the real platform administrator role. This role is intended for Kyle and future operators who can manage the Nabe installation itself.

`authenticated` is the baseline role for signed-in users without tenant membership. It allows basic Cloud DNS usage and own Device Client management without assigning the user to a tenant.

Current baseline global permissions:

- `platform.admin`
- `cloud_dns.use`
- `device_client.read_own`
- `device_client.manage_own`
- `engine.read`
- `engine.manage`

## Tenant Roles

Tenant roles are scoped to one tenant. The same user can be a `manager` in one tenant and only a `viewer` in another tenant.

Default tenant role templates:

- `manager`: tenant admin. Can manage tenant settings, members, Device Clients, and tenant query logs.
- `user`: normal tenant user. Can use tenant resources and manage own Device Clients.
- `viewer`: read-only member. Can inspect tenant resources and logs without making changes.

Tenant permissions are intentionally separate from global platform permissions:

- `tenant.read`
- `tenant.manage`
- `tenant.device_client.read`
- `tenant.device_client.manage`
- `tenant.querylog.read`

## Zitadel Mapping

Zitadel decides who can authenticate to the Nabe project. Nabe decides what an authenticated subject may do inside the product.

The intended mapping is:

- Zitadel project access allows login.
- A Nabe global role controls platform-wide abilities.
- A Nabe tenant membership controls tenant-specific abilities.
- Users without tenant membership can still use baseline Cloud DNS features through the `authenticated` global role.

For MVP 0, the existing backend session still returns the `admin` role after login. The multi-tenant schema exists so the next implementation step can move enforcement from this MVP shortcut to database-backed permission checks.

## Post-MVP Zitadel Automation

After MVP 0, Nabe can get a Zitadel service account to automate auth setup. This is not required for the MVP and should stay out of the critical path until login, dashboard, and Cloud DNS workflows are stable.

The service account idea:

- Create a Zitadel machine/service user for Nabe.
- Grant it only the Zitadel permissions needed to manage the Nabe project.
- Let Nabe create or verify the `nabe` project.
- Let Nabe create or verify the `nabe-web` OIDC application.
- Let Nabe configure redirect and post-logout URLs.
- Let Nabe create or verify project roles used for login claims.
- Keep Nabe's product permissions in the Nabe database, not exclusively in Zitadel.

This keeps Zitadel as the identity provider while leaving room for other auth providers later. A future provider adapter should normalize external claims into Nabe subjects, global roles, tenant memberships, and permissions.
