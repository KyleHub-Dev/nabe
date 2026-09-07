# Control-plane audit events

Nabe persists control-plane audit events in its embedded Turso database. This
is separate from DNS query logs: raw DNS history remains at the DNS engine by
default, while audit events record who accessed or changed Nabe resources.

`GET /api/audit/events` requires the `audit.read` permission. Platform admins
have this permission. Results are newest first and support these optional query
parameters:

- `actorSubject`: exact OIDC subject
- `action`: exact action name, such as `dns_policies.apply`
- `tenantId`: exact tenant ID when the audited resource is tenant-scoped
- `limit`: 1 to 500, default 100

The response never contains credentials, cookies, tokens, or raw DNS query
payloads. Reads of the audit event API are themselves appended as audit events.
Events have fields reserved for tenant, request ID, reason, outcome, and safe
structured metadata so tenant-scoped decisions and break-glass workflows can
be recorded without another schema redesign.
