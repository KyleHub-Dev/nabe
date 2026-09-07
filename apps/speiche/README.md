# Speiche

Speiche is Nabe's Apache-2.0 edge agent. It loads local configuration, enrolls with
the Nabe API and sends outbound heartbeats with edge and AdGuard status. The local
AdGuard driver handles engine operations. It opens no public server by default.

The CLI installs and configures Speiche on edge hosts. See the
[edge enrollment guide](../../docs/edge-enrollment.md) and
[CLI instructions](../cli/README.md).

Run the agent's checks from this directory:

```sh
go test ./...
```

This describes the working source. Enrollment and driver changes remain under
development; verify the intended edge workflow before deploying it.
