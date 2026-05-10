# AdGuard Home DNS Engine

AdGuard Home is the first DNS Engine for Nabe.

Nabe backend uses the AdGuard API. The browser must never receive AdGuard admin credentials. Native AdGuard UI should remain private and unmapped unless debugging.

The MVP should use AdGuard Persistent Clients and Allowed Clients to support tokenized Device Clients. ClientID tokens are the model for DoT, DoH, and DoQ access.

Query logs are consumed by Nabe and filtered server-side by ownership before being shown in the Nabe Console.
