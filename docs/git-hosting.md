# Git hosting

[GitHub](https://github.com/KyleHub-Dev/nabe) is Nabe's canonical repository.
`origin` fetches and pushes there. There is no automatic mirror.

Clone a new checkout:

```sh
git clone https://github.com/KyleHub-Dev/nabe.git
```

For an existing checkout, run `scripts/setup-git-remotes.sh` from its root. It
sets the canonical URL, clears explicit push URLs, and migrates tracking from the
old `github` convenience remote before removing it. It changes local Git
configuration only; it does not publish commits or change repository visibility.

Verify the result:

```sh
git remote get-url origin
git remote get-url --push --all origin
```

Both commands should return only `https://github.com/KyleHub-Dev/nabe.git`.
Store authentication in a credential helper, outside the repository.

The default CLI and Speiche downloads use GitHub source archives. Anonymous
installation requires the repository to be public. While access is restricted,
use an authorized checkout and build the CLI locally with `go build` in
`apps/cli`; use `NABE_SOURCE_URL` for an accessible source archive when installing
Speiche. The installer does not add authentication to archive requests.
