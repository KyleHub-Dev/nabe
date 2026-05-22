# ADR 0007: Local AdGuard Break-Glass Alias

Status: Accepted

Date: 2026-05-22

Supersedes: none

## Context

Tenant owners and admins may need emergency access to an edge AdGuard Home UI
without remembering an IP address. A local name such as `adguard.home` is useful
when debugging DNS or DHCP cutovers.

Native AdGuard UI cannot enforce Nabe's tenant model, audit model, or
permission filtering. It is therefore break-glass access, not normal product
UX.

## Decision

The edge installer may optionally publish a local DNS alias for the native
AdGuard Home UI.

The secure default remains:

- No LAN-visible native AdGuard UI.
- No `adguard.home` rewrite.
- Normal operations happen through Nabe.

When the operator passes `--adguard-ui-alias adguard.home`, the installer:

- Detects or accepts the edge LAN IP.
- Binds the AdGuard UI to that edge LAN IP and port.
- Generates credentials if none were provided.
- Stores the credentials root-only under `/etc/nabe/adguard-ui.env`.
- Adds an AdGuard DNS rewrite rule for the alias.

The rewrite uses AdGuard's DNS filtering `dnsrewrite` rule syntax.

## Consequences

The local network can resolve `adguard.home` to the edge IP if clients use the
edge AdGuard instance as DNS.

This is convenient, but it expands the attack surface. It must remain opt-in and
must use credentials.

## Security Considerations

This alias is not tenant-scoped by itself. Anyone on the local network who can
resolve and reach the UI can attempt login. Tenant authorization still belongs
in Nabe, not in native AdGuard UI.

For real multi-tenant or OT-like use, prefer private access paths and audited
Nabe operations over native UI exposure.

