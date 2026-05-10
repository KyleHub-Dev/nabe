# Git Remotes And Mirroring

Codeberg is primary. GitHub is a mirror for reach and community.

`origin` fetches from Codeberg:

```sh
git remote get-url origin
```

`origin` pushes to both Codeberg and GitHub:

```sh
git remote get-url --push --all origin
```

Expected setup:

```sh
git remote add origin ssh://git@codeberg.org/KyleHub/nabe.git
git remote set-url --push origin ssh://git@codeberg.org/KyleHub/nabe.git
git remote set-url --add --push origin https://github.com/KyleHub-Dev/nabe.git
git remote add github https://github.com/KyleHub-Dev/nabe.git
```

GitHub HTTPS push may require local credentials or a token configured by Git credential helpers. Do not store credentials in the repository.

Update remotes idempotently:

```sh
scripts/setup-git-remotes.sh
```
