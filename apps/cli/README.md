# Nabe CLI

The Go CLI installs and configures Nabe edge nodes. It can install AdGuard Home,
Unbound and Speiche, enroll an edge, and update an existing installation.
These operations modify host packages, configuration and system services.

## Build

From an authorized checkout, with Go 1.22 or later:

```sh
cd apps/cli
go build -o nabe .
./nabe version
./nabe status
```

For installation and enrollment, read the [edge guide](../../docs/edge-enrollment.md)
and the [README installation procedure](../../README.md#edge-install-guide).
Run host-changing commands only on the intended edge device.

Source downloads use the public GitHub repository. See
[Git hosting](../../docs/git-hosting.md) for local builds and source archive
configuration.
