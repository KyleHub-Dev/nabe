# Edge Enrollment

Planned flow:

1. Admin creates an Edge Node in Nabe.
2. Nabe issues an enrollment token.
3. Edge host runs Speiche with the token.
4. Speiche connects outbound to Nabe API.
5. Nabe validates the token and marks the node online.
6. Speiche sends heartbeats, inventory, and job results.

Optional Newt/Pangolin routes may later provide private break-glass access to native edge UIs. They are not the main control plane.
