# Security

- Do not run an open resolver.
- Do not expose public plain DNS/53 for Cloud DNS unless it is intentionally restricted.
- Prefer encrypted DNS with ClientID tokens for Cloud DNS: DoT, DoH, and DoQ.
- Store AdGuard API credentials server-side only.
- Never send AdGuard admin credentials to the browser.
- Filter user-facing query logs and stats server-side by owned client IDs.
- Keep native AdGuard UI private and use it only for break-glass/debug access.
- Edge Nodes should connect outbound through Speiche instead of requiring inbound ports.
- Do not commit secrets, tokens, passwords, filled `.env` files, private keys, or credentials.

Tailscale may be useful later as an optional Layer-3 mesh, but it is not a default dependency.
