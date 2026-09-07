# Licensing

Original Nabe source, documentation and repository tooling default to
AGPL-3.0-or-later. The platform components retain that license.

The CLI, Speiche and the current adapter/config/protocol packages use Apache-2.0
as listed in the root [LICENSE](../LICENSE). Each component's `LICENSE` file is
authoritative; full license texts live in `LICENSES/`.

Apache-licensed components must not import AGPL platform code. AGPL platform code
may depend on Apache components. Keep shared protocol and DNS-engine client code
separate from platform product logic. A narrower household focus does not change
these boundaries.

Third-party dependencies retain their own licenses. The AdGuard Home and Unbound
programs installed on an appliance have their own terms; Nabe's Apache-licensed
installer does not relicense them. License notices and corresponding-source
obligations must be handled for the actual artifacts distributed to a household.

The license grants no trademark rights or permission to imply KyleHub endorsement.
