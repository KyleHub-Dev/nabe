# Edge Enrollment

Enrollment flow:

1. Admin creates an Edge Node in Nabe.
2. The edge host runs Speiche with the explicitly dev-only bootstrap token.
3. Speiche connects outbound to `POST /api/edge/enroll`.
4. Nabe validates the bootstrap token and issues a random 256-bit credential
   bound to that node ID. The plaintext credential is returned once; central
   stores only its SHA-256 digest.
5. Speiche stores the credential in its root-only identity file and uses it for
   heartbeats and statistic batches. The bootstrap token is not accepted by
   those endpoints.
6. Nabe rejects another enrollment for the same node while its credential is
   active, preventing silent credential replacement.

An administrator can request controlled re-enrollment with
`POST /api/edge/nodes/{nodeId}/reenroll`. This revokes the current credential
and creates a durable audit event. When the next heartbeat receives a forbidden
response, Speiche preserves its stable node ID and pending telemetry, clears
the rejected credential, and uses the configured bootstrap token to enroll on
the following cycle.

`NABE_DEV_EDGE_TOKEN` remains a local-development bootstrap mechanism, not a
production enrollment-token lifecycle. It is accepted only by the enrollment
endpoint. Do not expose it publicly or reuse the development value in a real
deployment.

Current heartbeat defaults:

- Speiche sends a full node report every 30 seconds while central is reachable.
- After 3 consecutive failed central requests, Speiche backs off the next sleep
  by doubling it until a 300 second maximum.
- The backoff resets to 30 seconds after the next successful enrollment or
  heartbeat.
- Nabe derives `stale` and `offline` from `lastSeenAt` when edge nodes are
  listed. Defaults are `stale` after 3 expected heartbeat intervals and
  `offline` after 10.

The heartbeat report is for current node state: identity, host inventory, and
local health. Privacy-preserving statistic buckets use the separate
`/api/edge/stats` endpoint so regular heartbeats stay small and predictable.

Edge AdGuard UI defaults:

- `nabe install --edge` binds the AdGuard Home UI/API to
  `127.0.0.1:3000` by default.
- A local break-glass alias can be enabled with
  `--adguard-ui-alias adguard.home`. This adds a local AdGuard DNS rewrite,
  publishes the UI on the edge LAN IP, and stores generated credentials in
  `/etc/nabe/adguard-ui.env`.
- A LAN-visible UI requires an explicit bind such as
  `--adguard-ui-bind 0.0.0.0:3000`.
- Any non-loopback UI bind also requires `--adguard-admin-user` and
  `--adguard-admin-password`; the installer writes a bcrypt-backed AdGuard
  user. Loopback-only dev installs may omit users.

Edge DHCP defaults:

- `nabe install --edge` keeps AdGuard Home DHCP disabled by default.
- DHCP is enabled only with `--adguard-dhcp`.
- When DHCP is enabled, the installer requires:
  - `--adguard-dhcp-interface`, for example `eth0`
  - `--adguard-dhcp-gateway`, for example `10.0.0.1`
  - `--adguard-dhcp-subnet`, for example `255.255.255.0`
  - `--adguard-dhcp-range-start`, for example `10.0.0.100`
  - `--adguard-dhcp-range-end`, for example `10.0.0.250`

Router cutover for a home LAN:

Prefer keeping router DHCP and advertising the appliance as DNS when the router
supports that setup. For AdGuard DHCP, the installer enables DHCP when passed
`--adguard-dhcp`; it does not stage a disabled configuration for later review.

1. Record existing DHCP settings, reservations and a rollback procedure. Configure
   a stable Pi address outside the dynamic range or exclude it explicitly.
2. Check the interface, gateway, subnet and proposed lease range before running
   the DHCP-enabled installer. Account for IPv6 DNS advertisement separately.
3. Disable the previous DHCP server, then run the installer with the verified
   `--adguard-dhcp` settings. Use the Pi's stable address for continued access.
4. Renew one test client's lease. Verify its address, gateway, DNS server and
   actual DNS resolution and filtering before renewing the remaining clients.
5. If validation fails, disable AdGuard DHCP before restoring the previous DHCP
   server and settings. Renew the test client's lease and verify recovery.

Do not run independent DHCP servers on the same LAN as an improvised failover
pair. Different lease ranges do not coordinate DNS options, reservations or
lease ownership. If the router forwards all DNS requests itself, AdGuard may see
only the router address; do not claim per-device attribution in that setup.

Optional Newt/Pangolin routes may later provide private break-glass access to native edge UIs. They are not the main control plane.

Deferred central enrollment:

- `nabe install --edge --defer-enrollment` installs the local edge stack
  without writing `/etc/nabe/speiche.env` or starting Speiche. On an existing
  edge, it also stops and disables Speiche so a stale central URL is not used.
- Use this when the Pi should be prepared before the central API URL or
  enrollment token is final.
- Later, run:

```sh
nabe configure edge --remote "http://nabe-central.local:8080" --token "<edge-token>"
```

That writes `/etc/nabe/speiche.env`, enables Speiche, and restarts it so the
edge can enroll.

Reusing an existing enrollment during upgrades:

- `nabe install --edge --reuse-enrollment` re-runs the edge install while
  keeping the enrollment token already stored in the root-owned
  `/etc/nabe/speiche.env`.
- Add `--remote <url>` to point the existing enrollment at a new central API
  URL without re-entering the token.
- `--reuse-enrollment` cannot be combined with `--token` or
  `--defer-enrollment`.
