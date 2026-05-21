# ADR 0001: Architecture Decision Method

Status: Accepted

Date: 2026-05-21

Supersedes: none

## Context

Nabe has product notes, architecture notes, and security notes, but no canonical
decision record format. As the system grows into a central DNS control plane
with edge nodes, identity, tenant permissions, logs, stats, and security
boundaries, decisions need to be traceable and reviewable.

The project also needs a method for resolving tensions such as:

- One dashboard experience versus distributed edge data.
- Instant log viewing versus not centralizing raw DNS history.
- Operator convenience versus IT/OT safety boundaries.
- Fast development versus production-grade security defaults.

## Decision

Nabe will use ADRs for decisions that affect architecture, security,
permissions, data ownership, edge behavior, or operational method.

Each ADR must include:

- Status.
- Date.
- Supersedes, when applicable.
- Context.
- Decision.
- Consequences.
- Security considerations, when relevant.
- Implementation notes, when relevant.
- References, when external standards or docs shaped the decision.

Decision quality rules:

- Deny by default for access and data visibility.
- Prefer explicit resource ownership over implicit network trust.
- Prefer typed protocols over generic remote command execution.
- Prefer local raw-log retention over central raw-log accumulation.
- Separate interactive paths from durable background jobs.
- Treat DNS as operational infrastructure, not just app data.

## Consequences

Old docs may remain as explanation, but ADRs are authoritative once accepted.
Future implementation should update or create ADRs before introducing major
security or architecture changes.

Small tactical choices do not need ADRs. Cross-cutting changes do.
