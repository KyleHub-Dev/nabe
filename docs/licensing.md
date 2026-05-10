# Licensing

Nabe uses mixed licensing by package/component.

- Nabe platform components are AGPL-3.0-or-later.
- Speiche Agent is Apache-2.0.
- Shared protocol, SDK, and DNS Engine client libraries should use Apache-2.0 when they remain clean and do not contain platform/product logic.

Apache-licensed components must not import AGPL platform code. AGPL platform code may depend on Apache components.

Package-level `LICENSE` files are authoritative. The top-level `LICENSE` describes the mixed-license policy and points to canonical license texts in `LICENSES/`.
