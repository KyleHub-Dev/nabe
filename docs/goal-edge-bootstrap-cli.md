# Goal: Edge Bootstrap CLI

This file is written for Codex `/goal`.

Official Codex guidance for `/goal`: use one durable objective with a
verifiable stopping condition, name the validation loop, work in checkpoints,
and keep a short progress log. `/goal` is experimental and requires
`features.goals`; this workstation already has `goals = true` in
`~/.codex/config.toml`.

Source docs:

- https://developers.openai.com/codex/use-cases/follow-goals
- https://developers.openai.com/codex/cli/slash-commands

## Starter Prompt

```text
/goal Implement docs/goal-edge-bootstrap-cli.md without stopping until the CLI can bootstrap the real Raspberry Pi edge target and all listed verification checks pass, or until the run is blocked by a specific external prerequisite.
```

## Objective

Build the first end-to-end Nabe edge bootstrap path:

```sh
curl -fsSL https://codeberg.org/KyleHub/nabe/raw/branch/main/install.sh | bash
nabe install --edge --remote "http://10.0.0.230:8080" --token "<dev-token>"
```

The result must be a clean Raspberry Pi OS Lite machine turned into a working
Nabe Edge DNS node with Speiche, AdGuard Home, and a local upstream resolver.

## Real Target

Use this physical target for final validation:

```text
Host: 10.0.0.10
SSH user: kyle
Model: Raspberry Pi 3 Model B Plus Rev 1.3
OS: Debian GNU/Linux 13 (trixie), Raspberry Pi OS Lite
Kernel: 6.12.75+rpt-rpi-v8
Architecture: aarch64
CPU: 4x Cortex-A53
Memory: about 905 MiB RAM, about 904 MiB swap
Disk: 117G root, about 110G free on clean install
Network: eth0 10.0.0.10/24, wlan0 down
Available initially: curl, bash, sudo, systemctl
Missing initially: docker, podman
Sudo: passwordless
```

Do not commit SSH passwords, tokens, filled `.env` files, host keys, private
keys, or generated credentials.

## Current Dev Central

The local central API must be reachable from the Pi over the LAN. On this
workstation the Pi-facing IP is:

```text
10.0.0.230
```

For local validation, the Nabe dev stack should expose:

```env
NABE_API_BIND=0.0.0.0:8080
NABE_WEB_BIND=0.0.0.0:5173
NABE_PUBLIC_URL=http://10.0.0.230:5173
NABE_API_URL=http://10.0.0.230:8080
VITE_NABE_API_URL=http://10.0.0.230:8080
OIDC_REDIRECT_URI=http://10.0.0.230:8080/auth/callback
OIDC_POST_LOGOUT_REDIRECT_URI=http://10.0.0.230:5173/
```

The Pi must never use `localhost` for central. On the Pi, `localhost` means
the Pi itself.

## Required Product Behavior

Implement a minimal dev enrollment path first. Security-hardening can come
later, but dev-only behavior must be obvious in code and docs.

Required central behavior:

- A dev edge token can be configured through an environment variable.
- The web console or a simple authenticated/dev endpoint displays the dev token
  clearly enough for local testing.
- The API accepts edge enrollment/heartbeat requests using the dev token.
- The API stores or at least reports an enrolled edge node with:
  - node id
  - node name
  - hostname
  - architecture
  - OS/kernel
  - Speiche version
  - last seen timestamp
  - health status

Required CLI behavior:

- `install.sh` installs the `nabe` CLI on Raspberry Pi OS Lite.
- `nabe version` works after install.
- `nabe install --edge --remote <url> --token <token>` performs the edge
  bootstrap.
- The CLI validates that `--remote` is not `localhost` when running edge mode.
- The CLI detects OS, architecture, init system, network address, and available
  package tools.
- The CLI installs all missing runtime dependencies.
- The CLI installs and configures Speiche.
- The CLI installs and configures AdGuard Home.
- The CLI installs and configures a local upstream resolver, preferably
  Unbound unless a better local reason is documented.
- The CLI creates systemd services and enables them on boot.
- The CLI runs final health checks and exits non-zero if the edge is not
  actually usable.

Required Speiche behavior:

- Speiche starts as a systemd service.
- Speiche enrolls against the central API using `--remote` and `--token`.
- Speiche stores its local identity/config under a Nabe-owned path such as
  `/etc/nabe` or `/var/lib/nabe`.
- Speiche reports heartbeats to central.
- Speiche reports enough inventory for the central API to identify the Pi.
- Speiche can check local AdGuard and resolver health.

Required DNS behavior:

- The Pi listens for DNS on `10.0.0.10:53`.
- Normal DNS resolution works through the Pi.
- A known blocked test domain is blocked.
- AdGuard Home forwards upstream to the local resolver.
- Native AdGuard UI/API is not publicly routed by default.

## Explicit Non-Goals

Do not spend time on these unless they are necessary to complete the checks:

- Final website layout/content polish.
- Production-grade enrollment token lifecycle.
- Multi-edge fleet UX.
- Pi-hole migration.
- Pangolin/Newt route automation.
- Public native AdGuard UI.
- Device-client ownership UI.
- Query-log UI.
- Full central production deployment.

Optional Pangolin/Newt support may be left as a documented follow-up. The
first edge path is LAN-direct from Speiche to Nabe central.

## Suggested Checkpoints

Keep a short progress log in the final response and after major checkpoints.

1. Central dev readiness:
   - Add dev token config.
   - Add edge enrollment/heartbeat API.
   - Add a minimal way to retrieve/display the dev token.
   - Verify `curl http://10.0.0.230:8080/health` from the Pi.

2. CLI installer:
   - Add root `install.sh`.
   - Install/build `nabe` CLI for `linux/arm64`.
   - Verify `nabe version` on the Pi.

3. Edge bootstrap command:
   - Implement `nabe install --edge --remote ... --token ...`.
   - Detect target OS/arch/network.
   - Install missing dependencies.
   - Create Nabe config/state directories.

4. Edge services:
   - Install Speiche as a systemd service.
   - Install AdGuard Home.
   - Install and configure Unbound.
   - Ensure all services start on boot.

5. Enrollment and heartbeat:
   - Speiche enrolls with central.
   - Speiche reports heartbeat and inventory.
   - Central shows or returns the node as online.

6. DNS verification:
   - From the Pi, resolve a normal domain through local DNS.
   - From the workstation, resolve a normal domain through `10.0.0.10`.
   - Verify a known blocked domain is blocked.
   - Verify port 53 is listening on `10.0.0.10`.

7. Reboot verification:
   - Reboot the Pi.
   - Confirm SSH returns.
   - Confirm services are active.
   - Confirm central sees the node online again.
   - Confirm DNS still works.

## Verification Commands

Use focused checks while developing:

```sh
cd apps/cli && go test ./...
cd apps/speiche && go test ./...
env -u APPIMAGE -u APPDIR cargo check --manifest-path apps/api/Cargo.toml
pnpm --filter @nabe/web lint
pnpm --filter @nabe/web build
```

Use the real Pi for final checks. Replace the token with the dev token from
central:

```sh
ssh kyle@10.0.0.10 'curl -fsSL http://10.0.0.230:8080/health'
ssh kyle@10.0.0.10 'curl -fsSL https://codeberg.org/KyleHub/nabe/raw/branch/main/install.sh | bash'
ssh kyle@10.0.0.10 'nabe version'
ssh kyle@10.0.0.10 'nabe install --edge --remote "http://10.0.0.230:8080" --token "<dev-token>"'
ssh kyle@10.0.0.10 'systemctl is-active speiche'
ssh kyle@10.0.0.10 'systemctl is-active AdGuardHome || systemctl is-active adguardhome'
ssh kyle@10.0.0.10 'systemctl is-active unbound'
ssh kyle@10.0.0.10 'ss -lntup | grep ":53 "'
ssh kyle@10.0.0.10 'dig @127.0.0.1 example.com'
dig @10.0.0.10 example.com
```

If `dig` is not available, install `dnsutils` on Debian/Raspberry Pi OS or use
another DNS query tool and document the substitution.

## Stopping Condition

Stop only when one of these is true:

- The real Pi at `10.0.0.10` passes CLI install, edge bootstrap, enrollment,
  service health, DNS resolution, blocking, and reboot recovery checks.
- The run is blocked by one concrete external prerequisite that Codex cannot
  satisfy from the repository or over SSH, and the blocker is documented with
  the exact command/output that proves it.

