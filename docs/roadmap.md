# Household pilot roadmap

The target is the [household appliance experience](product.md). This roadmap
replaces the broad "everything working" goal as the next development scope.
Current-state observations below are from the working tree on 7 September 2026,
not a release or a verified household deployment.

## Existing foundation and gaps

| Area | Evidence in the working source | Needed for the pilot |
| --- | --- | --- |
| Provisioning | The CLI installs AdGuard Home, Unbound and Speiche and supports in-place reinstall. | Household-run Pi OS bootstrap, safe adoption of an existing installation and browser setup. |
| Pairing | Enrollment issues node-bound credentials, but starts with `NABE_DEV_EDGE_TOKEN`. | Authenticated household assignment with an expiring, single-use appliance claim and explicit re-pairing. |
| Connectivity | Speiche sends enrollment, heartbeats and aggregate statistics outbound. | Household-scoped management operations, application acknowledgements and retries. |
| Policy apply | `/api/dns-policies/apply` updates the central AdGuard instance. | Apply household policy to its selected appliance; a heartbeat does not implement this. |
| Authorization | Native roles, tenant resources and telemetry permissions are implemented in the source. | Prove isolation for pairing, appliance access, settings and statistics across all three households. |
| Recovery | The installer has an opt-in native AdGuard alias and stored credentials. | A small local Nabe recovery page and router rollback instructions usable without SSH. |

## 1. Establish the supported installation

Use the three pilot networks listed in [product.md](product.md). Record each Pi
model, OS release, router model/firmware and current addressing when available.
Unknown router details do not block installer or dashboard development.

The household installs Pi OS, connects Ethernet, sets a stable address and runs
the installer. On Kyle's existing Pi, first inventory and preserve configuration,
identity and data; use an adoption/upgrade path rather than treating it as blank.
Use no shared production bootstrap token.

Validate router-DHCP mode first where it supports the desired DNS configuration.
Also document and test the explicit AdGuard-DHCP mode, including range exclusions,
reservations, an ordered cutover and rollback. Verify DNS from an actual client
and check IPv6 separately. Do not enable a second independent DHCP server as a
fallback.

Complete when a fresh supported Pi OS installation can repeat the bootstrap, the
existing Pi has a verified adoption path, and each selected network mode has
working client checks and recovery instructions. Validate each household's router
settings before its cutover; a brand name alone is not a compatibility check.

## 2. Prove remote management before adding the wizard

Use an operator-assisted installation to pair appliances to the three
households. Send one allow/block change through central to the selected appliance,
receive an applied result, and verify the DNS response from a client on that LAN.
Reject pairing, reads and mutations attempted by either other household. Handle a
retry or disconnected appliance without duplicate effects or false success.

Complete when this path works without exposing an engine API or a general remote
shell. Keep the central/edge separation and constrained operations from the
accepted ADRs. Defer interactive log streaming until a pilot need justifies it.

## 3. Make setup and recovery usable by the household

Implement the browser wizard and local recovery path described in `product.md`.
Choose instructions by DNS/DHCP mode and record model-specific steps where
verified. Distinguish service health from verified network use. Test expired claims, interrupted setup and recovery access while
central and its identity provider are unreachable.

Complete when the household administrator runs the documented terminal bootstrap
and then performs wizard setup and routine changes without a terminal, with readable errors and a way back to the original router
settings. GitHub repositories remain private for now; provide an authorized
installer/source distribution path without embedding repository credentials in
the installer. Anonymous GitHub downloads are not yet a working bootstrap path.

## 4. Run the household pilot

Acceptance checks:

- The parents' and friend's households install Pi OS, run the installer, and
  complete the wizard and selected DNS/DHCP setup with the supplied instructions.
  Record every point requiring Kyle's help. Kyle's existing Pi passes adoption
  checks without losing its configuration or identity.
- From its own account, it checks status, blocks and allows a test domain, and
  pauses filtering. Verify the resulting client DNS behavior and pause expiry.
- Each household's account is denied access to either other household's appliance,
  settings and statistics through the API, not just hidden UI links.
- Disconnect central, then reboot the appliance. DNS still works with its last
  applied configuration. Local recovery remains available and a local pause
  expires without central. Reconnecting central reconciles status accurately.
- Power off the appliance and use the written network rollback to restore DNS.
  In appliance-DHCP mode, also verify a new lease and lease renewal after the
  documented DHCP recovery. Reconnect the appliance and verify the return to
  filtered DNS without competing DHCP servers.
- Simulate a changed router/subnet on a test network. Restore the appliance
  address, gateway and DNS/DHCP settings, retain pairing, and verify client DNS.
- Perform one operator-supervised upgrade using the documented installation
  path. Verify that identity, applied settings and recovery access survive, and
  record the recovery procedure before upgrading the other household.
- Run for seven days, record outages and support requests, and repeat the setup
  steps that caused confusion. This is a pilot observation window, not an uptime
  guarantee. Accept the pilot when blocking issues are resolved and the household
  can perform the routine actions without help.

After the pilot, evaluate two filtered DNS resolvers if households want automatic
continuity. Test loss of either resolver and policy consistency on both. Keep
coordinated DHCP failover separate. Choose other features from observed needs. The original
[MVP 0 plan](mvp-0.md) and [workstation goal](goal-everything-working.md) remain
historical references; neither adds requirements to this milestone.
