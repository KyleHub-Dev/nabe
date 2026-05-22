# Goal: Everything Working

This file is written for long-running coding-agent goal modes such as Codex
`/goal` and Claude Code `/goal`.

The purpose is to force one complete, verifiable Nabe flow instead of a UI demo
or disconnected service stubs. UI polish is explicitly secondary. The product
is considered working only when the central control plane on the workstation can
enroll, observe, and manage the Raspberry Pi edge node and DNS/device behavior
through Nabe-owned APIs and workflows.

## Objective

Build Nabe until the following single operator journey works end to end:

1. Start the Nabe central stack on this workstation.
2. Make central reachable from the Raspberry Pi over the LAN.
3. Enroll the Raspberry Pi as a Nabe Edge Node.
4. Run Speiche, AdGuard Home, and Unbound on the Pi as boot-enabled services.
5. Show the edge node online in central with inventory and health.
6. Manage edge, device-client, and DNS-policy state through central.
7. Apply the relevant DNS configuration to the edge DNS engine through Nabe.
8. Prove DNS resolution and blocking from both the Pi and workstation.
9. Restart central and reboot the Pi.
10. Prove the whole flow recovers and remains manageable.

## Real Topology

Do not treat any IP address, port, container state, or Pi service state in this
file as permanently true. The agent must discover the current topology at the
start of the goal and again before final verification.

Known historical/default topology:

```text
Repository: /home/kyle/KyleHub/nabe
Old workstation LAN IP used in earlier docs/env files: 10.0.0.230
Current observed workstation LAN IP on 2026-05-14: 10.0.0.101
Central API port: 8080
Central web port: 5173
```

Raspberry Pi edge target:

```text
Historical/current observed host on 2026-05-14: 10.0.0.10
SSH user: kyle
Model: Raspberry Pi 3 Model B Plus Rev 1.3
OS: Debian GNU/Linux 13 (trixie), Raspberry Pi OS Lite
Kernel family: 6.12.x+rpt-rpi-v8
Architecture: aarch64
Network: eth0 10.0.0.10/24, wlan0 down
Sudo: passwordless
```

The agent must compute the current Pi-facing central IP with commands such as:

```sh
ip -brief addr
ip route get "$NABE_EDGE_SSH_HOST"
```

Then the agent must verify the candidate central URL from both workstation and
Pi before using it:

```sh
curl -fsS "http://<current-workstation-lan-ip>:8080/health"
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'curl -fsS "http://<current-workstation-lan-ip>:8080/health"'
```

If `.env`, `.env.goal-edge.local`, Speiche config, docs, or commands contain a
stale central URL such as `http://10.0.0.230:8080`, update the local ignored
runtime files as needed and update tracked docs/scripts only when the stale
value is presented as authoritative rather than historical.

Do not commit SSH passwords, enrollment tokens, filled `.env` files, host keys,
private keys, generated credentials, or real secrets.

The ignored workstation file `.env.goal-edge.local` may contain SSH details for
manual verification. Source it only in the local shell:

```sh
set -a
. ./.env.goal-edge.local
set +a
```

Use `sshpass` only through `SSHPASS` when password SSH is needed:

```sh
SSHPASS="$NABE_EDGE_SSH_PASSWORD" sshpass -e ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'hostname'
```

## Central Runtime Requirements

The local central stack must be runnable from the repository with the existing
root `compose.dev.yaml` or a documented replacement that is equally simple.

For this goal, central must expose LAN-facing dev bindings. Replace
`<current-workstation-lan-ip>` with the value discovered at runtime:

```env
NABE_API_BIND=0.0.0.0:8080
NABE_WEB_BIND=0.0.0.0:5173
NABE_PUBLIC_URL=http://<current-workstation-lan-ip>:5173
NABE_API_URL=http://<current-workstation-lan-ip>:8080
VITE_NABE_API_URL=http://<current-workstation-lan-ip>:8080
OIDC_REDIRECT_URI=http://<current-workstation-lan-ip>:8080/auth/callback
OIDC_POST_LOGOUT_REDIRECT_URI=http://<current-workstation-lan-ip>:5173/
```

The Pi must never be configured to reach central through `localhost`.

## Current-State Discovery

The Pi and central stack may already be partially or mostly configured. The
agent must inspect and continue from the actual state rather than assuming a
clean machine.

At the start of the goal, collect and summarize:

- Current git branch and dirty files.
- Current workstation LAN addresses and route to the Pi.
- Current `.env` central URLs, without printing secrets.
- Current `.env.goal-edge.local` host/user/remote URL values, without printing
  passwords, tokens, or generated credentials.
- Current container status for `compose.dev.yaml`.
- Current central health/readiness from loopback and LAN IP.
- Current Pi reachability over SSH.
- Current Pi `nabe version`.
- Current Pi service state for Speiche, AdGuard Home, and Unbound.
- Current Speiche remote URL and identity state, if present.
- Current central `/api/edge/nodes` state.
- Current DNS behavior through the Pi.

Observed state on 2026-05-14 before this file was updated:

```text
Workstation LAN IP: 10.0.0.101
Old configured central URL in .env and .env.goal-edge.local: http://10.0.0.230:8080
Central health on 127.0.0.1:8080: ok
Central health on 10.0.0.101:8080: ok
Central health on 10.0.0.230:8080: timed out
Compose services: nabe-api, nabe-web, dns-adguard, dns-unbound up
Pi SSH host: 10.0.0.10
Pi hostname: raspberrypi
Pi eth0: 10.0.0.10/24
Pi nabe version: nabe cli 0.0.0
Pi services: speiche active, AdGuardHome active, unbound active
Central edge node edge-raspberrypi: persisted but reported offline
DNS through Pi for example.com: resolves
DNS through Pi for blocked.nabe.test: returns 0.0.0.0
```

This means the likely first repair is not a full fresh bootstrap. It is to
reconcile stale central URLs, Speiche config/heartbeat, and central online state
while preserving the already working Pi DNS services.

## Definition Of Everything Working

Everything working means Nabe is the control plane. Native AdGuard Home UI/API
may be used for break-glass debugging, but the verified operator flow must use
Nabe APIs, CLI, Speiche, or Nabe Console for normal control.

Required central behavior:

- Health and readiness endpoints report central state.
- Dev enrollment token exists for local testing and is clearly dev-only.
- Edge enrollment and heartbeat endpoints validate the token.
- Edge nodes are persisted and listable after central restart.
- Edge node status includes node id, name, hostname, architecture, OS/kernel,
  Speiche version, last seen timestamp, health status, and inventory.
- Central derives stale/offline state from heartbeat age.
- Central can expose edge details to the web console or a documented API/CLI.
- Central owns device-client records for the MVP.
- Central owns DNS policy records for the MVP.
- Central can apply the MVP DNS policy/client state to the active DNS engine.
- Browser-delivered data never contains AdGuard admin credentials.

Required edge behavior:

- `install.sh` installs the `nabe` CLI on Raspberry Pi OS Lite.
- `nabe version` works on the Pi.
- `nabe install --edge --remote <central-url> --token <token>
  --adguard-ui-alias adguard.home` bootstraps the Pi or reports an actionable
  failure.
- Edge install rejects `localhost`, `127.0.0.1`, and `::1` central URLs.
- Edge install detects OS, architecture, init system, network address, and
  package tool.
- Speiche runs as a systemd service and is enabled on boot.
- AdGuard Home runs as a systemd service and is enabled on boot.
- Unbound or the chosen local resolver runs as a systemd service and is enabled
  on boot.
- Speiche enrolls with central and sends recurring heartbeats.
- Speiche stores identity/config under Nabe-owned paths such as `/etc/nabe` and
  `/var/lib/nabe`.
- Speiche reports local DNS engine and resolver health.
- The Pi listens for DNS on its currently verified LAN address, historically
  `10.0.0.10:53`.

Required device-client behavior:

- A device client can be created, listed, inspected, updated, and disabled
  through Nabe-owned API/CLI/UI behavior.
- Device client records include a stable id, human label, owner or tenant scope
  placeholder, client token or ClientID value, enabled/disabled state, created
  timestamp, and updated timestamp.
- Disabled device clients must not be treated as active by policy application.
- Device client state survives central restart.

Required DNS policy behavior:

- A policy can be created, listed, inspected, updated, and applied through
  Nabe-owned API/CLI/UI behavior.
- The MVP policy must support at least one deterministic blocked domain used for
  verification, for example `blocked.nabe.test`.
- Applying policy updates the DNS engine used by the edge without requiring
  manual native AdGuard UI edits.
- Normal DNS resolution works through the edge.
- The known blocked test domain is blocked through the edge.
- The policy/device behavior is verified from the workstation against the
  currently verified Pi LAN address, not only from inside containers.

Required web behavior:

- The console can load against the LAN-facing API.
- The console shows central health/session state.
- The console shows Cloud DNS or engine status.
- The console shows enrolled edge nodes or links clearly to the API/CLI command
  that lists them.
- The console shows enough device/policy state to prove it is not a placeholder
  only. Visual design polish is not part of this goal.

## Explicit Non-Goals

Do not spend goal time on these unless they are necessary for verification:

- Final visual design polish.
- Marketing pages.
- Production enrollment token lifecycle.
- Production-grade multi-tenant RBAC.
- Query-log privacy UI beyond preventing credential leaks.
- Pi-hole migration.
- Pangolin/Newt route automation.
- Public native AdGuard UI exposure.
- Full production deployment hardening.

## Checkpoints

Keep a short progress log in final responses and after major checkpoints. For
Claude Code, print enough command output summaries that the goal evaluator can
judge progress from the transcript.

1. Baseline and branch:
   - Confirm git status.
   - Create or confirm a purpose branch.
   - Read `AGENTS.md`, `README.md`, and relevant docs.
   - Identify existing implementation before editing.

2. Central dev runtime:
   - Discover the current workstation LAN IP and route to the Pi.
   - Discover the current Pi IP and reachable SSH target.
   - Inspect central stack, Pi services, Speiche state, and DNS behavior.
   - Identify stale URLs or mismatched runtime config.
   - Prefer repairing current state over reinstalling working Pi services.
   - Configure LAN-facing local `.env` without committing secrets.
   - Start central stack.
   - Verify workstation `curl http://127.0.0.1:8080/health`.
   - Verify workstation `curl http://<current-workstation-lan-ip>:8080/health`.
   - Verify Pi `curl http://<current-workstation-lan-ip>:8080/health`.

3. Central data model and APIs:
   - Finish edge persistence if incomplete.
   - Add device-client persistence and endpoints.
   - Add DNS-policy persistence and endpoints.
   - Add engine apply behavior for the MVP policy/device state.
   - Add focused API tests or command-level checks.

4. CLI and edge bootstrap:
   - Ensure root `install.sh` works on Pi.
   - Ensure `nabe version` works on Pi.
   - Ensure `nabe install --edge --remote ... --token ...` installs and
     configures Speiche, AdGuard Home, and the local resolver.
   - Ensure repeated bootstrap repairs stale central URLs without damaging
     already working DNS services.
   - Ensure repeated bootstrap is idempotent enough for development.

5. Speiche and central heartbeat:
   - Verify Speiche enrolls.
   - Verify central lists the Pi as online.
   - Verify central receives current inventory and health.
   - Verify stale/offline behavior can be reasoned about or tested.

6. Device and policy management:
   - Create a test device client through Nabe.
   - Create or update the test DNS policy through Nabe.
   - Apply policy to the edge DNS engine through Nabe.
   - Confirm disabled clients/policies are handled as documented.

7. Web console proof:
   - Build the web app.
   - Start the web app against LAN-facing API.
   - Verify the console displays real API-backed central, edge, device, and
     policy state. Minimal UI is acceptable.

8. DNS proof:
   - From Pi, resolve `example.com` through `127.0.0.1`.
   - From workstation, resolve `example.com` through the verified Pi LAN
     address.
   - From Pi, query `blocked.nabe.test`.
   - From workstation, query `blocked.nabe.test` through the verified Pi LAN
     address.
   - Confirm the blocked domain result is deterministic and documented.

9. Restart and reboot proof:
   - Restart central stack.
   - Confirm edge/device/policy state persists.
   - Reboot the Pi.
   - Confirm SSH returns.
   - Confirm Speiche, AdGuard Home, and resolver services return active.
   - Confirm central sees the edge online again.
   - Confirm DNS still resolves and blocks correctly.

10. Final gate:
    - Run focused tests for changed Rust, Go, and TypeScript packages.
    - Run full repo verification if feasible.
    - Document any skipped check with the exact reason.

## Verification Commands

Focused local checks:

```sh
env -u APPIMAGE -u APPDIR cargo check --manifest-path apps/api/Cargo.toml
cd apps/cli && go test ./...
cd apps/speiche && go test ./...
pnpm --filter @nabe/web lint
pnpm --filter @nabe/web build
```

Full gate when feasible:

```sh
pnpm lint
pnpm build
pnpm test
pnpm verify
```

Central local checks:

```sh
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8080/ready
```

The dev edge token and edge-node inventory endpoints require an authenticated
platform-admin session. For edge bootstrap, read the configured token from the
central environment or secret store rather than exposing it unauthenticated over
HTTP.

Pi-to-central checks:

```sh
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'curl -fsS http://<current-workstation-lan-ip>:8080/health'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'curl -fsS http://<current-workstation-lan-ip>:8080/ready'
```

Pi bootstrap checks:

```sh
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'curl -fsSL http://<current-workstation-lan-ip>:8080/health'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'curl -fsSL https://codeberg.org/KyleHub/nabe/raw/branch/main/install.sh | bash'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'nabe version'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'nabe install --edge --remote "http://<current-workstation-lan-ip>:8080" --token "<dev-token>" --adguard-ui-alias adguard.home'
```

Pi service checks:

```sh
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'systemctl is-enabled speiche'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'systemctl is-enabled AdGuardHome || systemctl is-enabled adguardhome'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'systemctl is-enabled unbound'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'systemctl is-active speiche'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'systemctl is-active AdGuardHome || systemctl is-active adguardhome'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'systemctl is-active unbound'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'ss -lntup | grep ":53 "'
```

DNS checks:

```sh
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'dig +short @127.0.0.1 example.com'
ssh "$NABE_EDGE_SSH_USER@$NABE_EDGE_SSH_HOST" 'dig +short @127.0.0.1 blocked.nabe.test A'
dig +short @"$NABE_EDGE_SSH_HOST" example.com
dig +short @"$NABE_EDGE_SSH_HOST" blocked.nabe.test A
```

If `dig` is unavailable, install `dnsutils` on Debian/Raspberry Pi OS or use a
documented equivalent.

## Final End-To-End Acceptance Gate

The goal is complete only when the agent can report concrete evidence for all
of these:

- Central API and web are running on the workstation and reachable from the Pi.
- A dev enrollment token is configured without committing secrets.
- The Pi has `nabe` CLI installed.
- `nabe install --edge` has completed successfully against
  the currently verified workstation LAN API URL.
- Speiche, AdGuard Home, and Unbound or chosen resolver are enabled and active.
- Central lists the Pi edge as online with current inventory.
- A test device client exists in central.
- A test DNS policy exists in central.
- Applying policy through Nabe updates edge DNS behavior.
- `example.com` resolves through the Pi from both Pi and workstation.
- `blocked.nabe.test` is blocked through the Pi from both Pi and workstation.
- Central restart preserves edge, device, and policy state.
- Pi reboot preserves service health and central heartbeat recovery.
- Focused tests for changed packages pass.

## Stopping Condition

Stop only when one of these is true:

- The real Pi target discovered from `.env.goal-edge.local` or current network
  inspection and the currently verified workstation central LAN URL pass the
  final end-to-end acceptance gate.
- The run is blocked by one concrete external prerequisite that the agent cannot
  satisfy from the repository, workstation, or SSH access, and the blocker is
  documented with the exact command/output that proves it.

Do not stop merely because the implementation is large, because UI polish is
unfinished, or because some follow-up production hardening remains outside the
explicit scope.
