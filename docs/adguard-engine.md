# AdGuard Home DNS Engine

AdGuard Home is the first DNS Engine for Nabe.

Nabe backend uses the AdGuard API. The browser must never receive AdGuard admin credentials. Native AdGuard UI should remain private and unmapped unless debugging.

The MVP should use AdGuard Persistent Clients and Allowed Clients to support tokenized Device Clients. ClientID tokens are the model for DoT, DoH, and DoQ access.

Query logs are consumed by Nabe and filtered server-side by ownership before being shown in the Nabe Console.

## Compose Development Defaults

The checked-in development config at
`data/config/adguard/AdGuardHome.yaml` is used by `compose.dev.yaml`.

For local development it points AdGuard at the internal Unbound service first:

```text
upstream_dns: 172.30.10.10:53
bootstrap_dns: []
local_ptr_upstreams: 172.30.10.10:53
```

It also enables a bounded resolver cache:

```text
cache_ttl_max: 3600
cache_optimistic: true
cache_optimistic_answer_ttl: 30s
cache_optimistic_max_age: 12h
```

Development and edge configs use `https://1.1.1.1/dns-query` as fallback DNS
when the local Unbound resolver does not respond within five seconds. Unbound
remains the sole primary resolver, and strict DNSSEC behavior is retained by
both the local resolver and the validating public fallback.
