# Speiche Agent

Speiche is the generic Edge Node connector for Nabe. It is licensed Apache-2.0.

Speiche is not AdGuard-specific in concept. The MVP driver targets AdGuard Home because AdGuard Home is the first DNS Engine. Future drivers may support Blocky, Technitium, RethinkDNS, or other engines.

Responsibilities:

- Enrollment with Nabe using an enrollment token.
- Outbound heartbeat to Nabe API.
- Local DNS Engine inventory.
- Local stats collection.
- Job polling.
- Future config sync.

Speiche exposes no public server by default.
