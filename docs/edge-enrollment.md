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

Optional Newt/Pangolin routes may later provide private break-glass access to native edge UIs. They are not the main control plane.
