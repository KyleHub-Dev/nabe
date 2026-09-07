# Authorization Model

Status: Background.

Canonical authorization decisions now live in
`docs/adr/0003-authorization-permission-model.md`. If this document conflicts
with that ADR, the ADR supersedes it.

Nabe uses any standards-compliant OIDC provider, including Zitadel, for
authentication and Nabe-owned authorization data for product permissions.
Successful login proves identity only. Every protected request resolves current
global roles, tenant memberships, and permissions from the embedded database.
No role or permission is trusted from the browser session cookie.

## Concepts

- Subject: a signed-in identity from an external provider, currently Zitadel OIDC.
- Global role: a platform-wide role that is not bound to a tenant.
- Tenant: an isolated customer, family, team, or organization scope.
- Tenant group: a group-like external assignment key, for example `tenant_gray`.
- Tenant role: a role granted to a subject inside one tenant.
- Permission: a fine-grained action Nabe can check before returning data or changing state.
- Device Client: a DNS client owned by a subject and optionally attached to a tenant.

## Global Roles

`admin` is the real platform administrator role. This role is intended for Kyle and future operators who can manage the Nabe installation itself.

`baseline` is the implicit baseline role for signed-in users without tenant membership. It allows basic Cloud DNS usage and own Device Client management without assigning the user to a tenant.

Current `admin` permissions:

- `platform.admin`
- `cloud_dns.use`
- `device_client.read_own`
- `device_client.manage_own`
- `engine.read`
- `engine.manage`

Current `baseline` permissions:

- `cloud_dns.use`
- `device_client.read_own`
- `device_client.manage_own`

## Tenant Roles

Tenant authorization has two parts:

- Tenant group: which tenant the user belongs to, for example `tenant_gray`.
- Tenant role: what the user can do inside that tenant, for example `manager`, `user`, or `viewer`.

This keeps family/team membership separate from permission level. The same user can be a `manager` in `tenant_gray` and only a `viewer` in another tenant.

Nabe stores tenant groups as identity-provider mappings. For Zitadel, `tenant_gray` can map to the internal tenant with slug `gray`. Other auth providers can later provide equivalent group keys without changing Nabe's tenant tables.

Tenant group keys should use the `tenant_<slug>` pattern:

```text
tenant_gray
tenant_smith
tenant_kylehub
```

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
- A tenant group assigns a subject to a tenant.
- A Nabe tenant membership controls tenant-specific abilities.
- Users without tenant membership can still use baseline Cloud DNS features through the implicit `baseline` role.

Recommended role and group list:

| Key | Display Name | Group |
| --- | --- | --- |
| `admin` | Platform Admin | Global |
| `baseline` | Baseline User | Global Internal |
| `tenant_<slug>` | Tenant Group | Tenant Group |
| `manager` | Tenant Manager | Tenant |
| `user` | Tenant User | Tenant |
| `viewer` | Tenant Viewer | Tenant |

`baseline` is internal and does not need to be created as a Zitadel role. It is listed here because Nabe uses it as the effective baseline permission set after successful login.

Example for the Gray family:

| Key | Display Name | Group |
| --- | --- | --- |
| `tenant_gray` | Gray | Tenant Group |
| `manager` | Tenant Manager | Tenant |
| `user` | Tenant User | Tenant |
| `viewer` | Tenant Viewer | Tenant |

Fresh identities receive only the native `baseline` global role. Initial
operators must be explicitly listed as `<issuer-host>:<OIDC-subject>` in
`NABE_BOOTSTRAP_ADMIN_SUBJECTS`; after their verified login Nabe grants the
native `admin` role. Existing pre-native installations are migrated by
preserving subjects that were already stored as admins.

Platform admins can create tenants with `POST /api/tenants`. Nabe creates the
manager, user, and viewer roles and copies their permission templates into the
new tenant. Platform admins and members with `tenant.manage` can grant an
existing OIDC subject a tenant role through
`POST /api/tenants/{tenantId}/members`. `GET /api/tenants` returns every tenant
to platform admins and only assigned tenants to other users.

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
