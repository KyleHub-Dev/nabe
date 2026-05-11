# Nabe AdGuard Adapter

Apache-2.0 Rust adapter for the AdGuard Home `/control` API.

This package is the engine bridge layer for Nabe and Speiche. It deliberately lives outside the AGPL API application so the engine integration can remain permissively licensed and reusable by a future Speiche agent.

Current shape:

- `ALL_OPERATIONS` mirrors the AdGuard Home OpenAPI operation surface.
- `AdguardAdapter::raw_json` can call any listed operation.
- Typed convenience methods exist for the status, DNS info, stats, TLS status, and query-log endpoints currently used by Nabe.

Do not copy GPL implementation code from `reference/AdGuardHome` into this package. Use the public OpenAPI contract and runtime behavior as the source of truth.
