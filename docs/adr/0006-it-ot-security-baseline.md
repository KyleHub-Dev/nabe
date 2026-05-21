# ADR 0006: IT/OT Security Baseline

Status: Accepted

Date: 2026-05-21

Supersedes: previous informal security guidance in `docs/security.md` where it
conflicts with this ADR.

## Context

Nabe is currently a homelab/self-hosted DNS control plane, but DNS is
operational infrastructure. If used by a household, team, small business, or
OT-like network, broken DNS can disrupt work. If used around OT devices, unsafe
active discovery, uncontrolled policy changes, or remote management can create
real operational risk.

The design should borrow IT/OT security principles now, even if the first
deployment is personal.

## Decision

Nabe adopts these security baselines:

- Deny by default.
- Least privilege by role, tenant, owner, and resource.
- Explicit zones and conduits.
- Outbound-only edge management by default.
- No arbitrary remote shell through Speiche.
- No public native AdGuard UI by default.
- No public open resolver defaults.
- No active network scanning by default.
- Local DNS continuity if central is down.
- Versioned, validated, reversible policy changes.
- Per-command and per-sensitive-read audit.
- Signed production artifacts before production edge installer use.
- Secrets never sent to the browser.
- Raw DNS logs local by default.

Speiche is a constrained executor. Nabe central is a policy decision point and
policy administration point. Enforcement occurs both centrally and at the edge.

## Consequences

Nabe will sometimes choose a slower implementation path to preserve safety. For
example, policy apply needs preflight and rollback instead of direct blind
mutation.

The development installer can remain convenient, but production installation
must move toward signed artifacts and verifiable provenance.

## Security Considerations

Mandatory controls before real multi-tenant use:

- Deny-by-default authorization middleware on every API route.
- Stable tenant and Device Client ownership model.
- Tests for cross-tenant denial.
- Signed edge identity and revocation.
- Typed Speiche command protocol.
- Audit events for raw log views, policy changes, break-glass, and edge actions.
- Config drift detection between Nabe desired state and engine actual state.
- Backup and restore tests for central DB and edge identity/config.
- Explicit raw-log retention settings.

OT-specific constraints if Nabe ever manages DNS near operational networks:

- No automatic changes during operational windows without approval.
- No active inventory scans by default.
- No public cloud dependency for already-applied local DNS continuity.
- Separate approval for DHCP, resolver, or network-wide DNS cutovers.
- Change records must be attributable, reviewable, and reversible.

## References

- NIST SP 800-82 Rev. 3, Guide to Operational Technology Security.
- NIST SP 800-207, Zero Trust Architecture.
- NIST Cybersecurity Framework 2.0.
- CISA Secure by Design guidance.
- OWASP Logging Cheat Sheet.
