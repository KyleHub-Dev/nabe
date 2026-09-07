# Household DNS appliance

Nabe helps a household set up a Raspberry Pi for DNS filtering and manage it
through a browser dashboard. The first pilot covers three households: Kyle's
apartment, his parents' home and a friend's family home.

This is the current product direction, agreed in September 2026. The browser
wizard, household setup and recovery experience below are requirements, not
features already shipped. [roadmap.md](roadmap.md) owns implementation order and
pilot acceptance. [CONTEXT.md](../CONTEXT.md) defines the product language.

## Who it serves

The household administrator buys the Pi, installs a supported Raspberry Pi OS
release, connects Ethernet and runs the Nabe installer in a terminal. They can
configure a stable LAN address and DHCP. After bootstrap, the wizard and dashboard
should handle setup and routine use without requiring shell commands.

Kyle operates the shared Nabe service and supports the pilot. Central must be
reachable from all homes over authenticated HTTPS; it cannot depend on a
workstation being awake. One household administrator per home is sufficient
initially. Keep households' permissions and data separate without a role editor.

Known pilot networks, reported in September 2026:

| Household | Router | Starting point |
| --- | --- | --- |
| Kyle's apartment | Netgear, model not yet recorded | A Pi already runs on this network; preserve its configuration and data before adoption. |
| Parents' home | FRITZ!Box, model not yet recorded | Household-installed Pi OS and Nabe installer. |
| Friend's family home | Unknown and may change | Household-installed Pi OS and Nabe installer. |

Router brands are context, not verified compatibility claims. Record model,
firmware and DNS/DHCP capabilities during each installation. Support setup by
capability so a router replacement does not require a new appliance identity.

Use the existing tenant model for household access and the existing Edge Node
model for an appliance. These are product labels, not a request to rename the
schema or add another ownership hierarchy.

## First setup path

The household starts with Pi OS on its own hardware and working Ethernet. The
installer checks the OS, architecture, network and existing DNS/DHCP services,
then prepares the local stack and prints a browser setup address. Existing
installations need an adoption or upgrade path that preserves identity and data;
the installer must not silently replace another working DNS/DHCP configuration.

The household configures a stable address, either in Pi OS or through a router
reservation. The wizard verifies the address, subnet and gateway. A manually
assigned address must be outside the dynamic pool or explicitly excluded from it.
Changing the Pi's network configuration automatically is not required for the
first installer. A custom OS image and Wi-Fi provisioning are outside the pilot.

The wizard guides the household through these steps:

1. Identify the appliance and check local DNS, storage and central connectivity.
2. Sign in to central and pair the appliance to the intended household through a
   short-lived, single-use claim. A code alone must not allow pairing to another
   household; central verifies the signed-in user's access and appliance claim.
3. Explain the selected filtering defaults and how to pause them temporarily.
4. Verify the stable appliance address and choose who provides DHCP. Follow the
   matching DNS-advertisement or DHCP-cutover instructions. Verify one test
   device before applying the change to the household.
5. Prove that a test device uses the appliance and that filtering works. Save the
   dashboard address and a local recovery address with its authentication details.

Installing DNS software does not change the network's DNS configuration. The
wizard must distinguish "appliance ready" from "household using it". Account for
IPv6 DNS announcements and explain that applications using their own encrypted
DNS can bypass the household setting. Do not claim every device is protected
based only on a heartbeat or a successful query issued on the Pi itself.

Offer two explicit network modes:

- Router DHCP: keep the router issuing leases and configure it to advertise the
  appliance as DNS where supported. Prefer this path when it meets the household's
  needs; the router continues issuing leases if the Pi fails.
- Appliance DHCP: use AdGuard Home DHCP when the household chooses it or the router
  cannot advertise suitable DNS settings. Gather the interface, subnet, gateway,
  lease range and existing reservations. Plan the cutover and rollback before
  changing either server. Disable the previous DHCP server before enabling the
  appliance's server. Two independent DHCP servers are not a failover setup,
  even with different lease ranges.

Check IPv6 DNS and router advertisements separately; moving DHCPv4 does not
establish control of IPv6 DNS. Preserve the router's role as the network gateway.
When a router or subnet changes, provide instructions to restore the appliance's
address, gateway and DNS advertisement, then repeat client verification. Preserve
household pairing unless the administrator explicitly transfers the appliance.

## Availability and failover

| Failure | Required behavior or limitation |
| --- | --- |
| Central or identity provider unavailable | The Pi keeps serving local DNS with its last applied settings. |
| DNS service process fails | Supervise and restart it locally; report repeated failures. This is service recovery, not hardware redundancy. |
| Pi loses power or fails | A single Pi cannot provide DNS. Give the household offline router rollback instructions. |
| Pi provides DHCP and fails | Existing leases may continue temporarily, but new and renewing clients can lose configuration. DNS and DHCP recovery need separate checks. |

Automatic filtered DNS continuity is a follow-up goal. It normally needs two
reachable resolvers, such as two local appliances, with the same intended policy.
Clients may use either advertised DNS server at any time; "secondary" does not
mean an idle standby. Both must enforce the household's settings. A public
unfiltered secondary may bypass filtering even when the Pi is healthy.

For that follow-up, prefer retaining router DHCP and making DNS redundant first.
DHCP failover requires coordinated lease ownership and recovery; copying AdGuard
configuration alone does not provide it. Do not add automatic router-DHCP toggles
or claim high availability for the single-Pi pilot. Expose the chosen availability
tradeoff during setup instead of presenting an unfiltered fallback as equivalent
protection.

## Routine dashboard

The household administrator needs to see whether the appliance is reachable,
whether DNS is healthy, and when settings last applied. They can enable a small
curated default filter set, add an allowed or blocked domain, and pause filtering
for a bounded period with automatic resumption.

Show pending, applied and failed policy changes separately. A successful database
write is not proof that the appliance changed. Show aggregate household activity
only after access checks; device names and device-level views require verified
client identity. If the router hides clients behind its address, label the data
as network-level rather than inventing device attribution.

Detailed live query browsing, per-person policy assignment and a customizable
permission system can wait. They are not prerequisites for this household to use
DNS filtering.

## Failure and recovery

The appliance continues serving DNS with its last applied settings when Nabe or
the identity provider is unavailable, including after an appliance reboot. This
does not promise resolution of uncached external names during an Internet outage.
Central being offline must not require a public unfiltered DNS fallback.

A proposed local Nabe recovery page provides DNS and connection status, a bounded
filtering pause, and instructions for restoring the previous DNS/DHCP setup.
It must work without central login, use appliance-local authentication, and remain
reachable by a recorded numeric LAN address when DNS lookup is broken. Initial
claim access expires after setup; ongoing recovery access is not an unauthenticated
LAN admin page. A local pause must expire locally and become visible to central
when connectivity returns.

A powered-off or failed Pi cannot serve a recovery page. Provide the router
rollback instructions separately so the household can restore connectivity from
a phone. One Pi provides no hardware redundancy.

SSH and the opt-in native AdGuard interface remain operator recovery tools.
Speiche uses outbound authenticated connections; the household opens no inbound
router ports. Keep remote operations typed and constrained rather than adding
arbitrary shell access. The existing security ADRs still apply.

## Scope boundary

Ship a documented Pi OS bootstrap, one appliance per household, AdGuard Home plus
Unbound, a setup wizard with explicit DNS/DHCP modes, basic filtering, honest
status and recovery. Validate against the three pilot networks as they are
identified. Keep the existing CLI as the installation and operator tool.

Defer a public Cloud DNS offering, roaming-device clients, additional DNS engine
adapters, automatic router reconfiguration,
Wi-Fi onboarding, an image updater, billing, enterprise/OT workflows and a generic
remote-management system. Preserve existing code and accepted security decisions;
these capabilities do not expand the first release's acceptance criteria. A
second filtered resolver is the first availability follow-up if the pilot
households want automatic DNS continuity; coordinated DHCP failover is separate.
