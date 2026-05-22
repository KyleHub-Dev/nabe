# Edge Enrollment

Planned flow:

1. Admin creates an Edge Node in Nabe.
2. Nabe issues an enrollment token.
3. Edge host runs Speiche with the token.
4. Speiche connects outbound to Nabe API.
5. Nabe validates the token and marks the node online.
6. Speiche sends heartbeats, inventory, and job results.

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
local health. Logs, DNS query data, and richer stats should use separate
endpoints or jobs so regular heartbeats stay small and predictable.

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

1. Keep the router DHCP server enabled.
2. Run the edge installer with `--adguard-dhcp` and a non-overlapping range.
3. Open the AdGuard UI and verify DHCP settings are present but do not switch
   clients yet if the router is still serving the same range.
4. Disable DHCP on the router.
5. Restart WiFi on one client or renew its lease.
6. Confirm the client receives DNS server `10.0.0.10` directly.
7. Confirm AdGuard query log shows the real client IP instead of only the
   router IP.

Do not run two DHCP servers on the same LAN range. During migration, use a
small test range or switch router DHCP off immediately after enabling AdGuard
DHCP.

Optional Newt/Pangolin routes may later provide private break-glass access to native edge UIs. They are not the main control plane.

Deferred central enrollment:

- `nabe install --edge --defer-enrollment` installs the local edge stack
  without writing `/etc/nabe/speiche.env` or starting Speiche.
- Use this when the Pi should be prepared before the central API URL or
  enrollment token is final.
- Later, run:

```sh
nabe configure edge --remote "http://10.0.0.230:8080" --token "<edge-token>"
```

That writes `/etc/nabe/speiche.env`, enables Speiche, and restarts it so the
edge can enroll.
