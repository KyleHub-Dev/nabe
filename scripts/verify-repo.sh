#!/usr/bin/env sh
set -eu

echo "== git remotes =="
git remote -v

echo "== required files =="
for path in \
  README.md \
  LICENSE \
  LICENSES/AGPL-3.0-or-later.txt \
  LICENSES/Apache-2.0.txt \
  .env.example \
  compose.dev.yaml \
  apps/web/package.json \
  apps/api/Cargo.toml \
  apps/worker/package.json \
  apps/speiche/go.mod \
  apps/cli/go.mod \
  packages/validation/package.json \
  packages/edge-protocol/LICENSE \
  packages/config/LICENSE; do
  test -e "$path" || { echo "missing $path"; exit 1; }
done

echo "== package license sanity =="
grep -R "Apache-2.0\\|AGPL-3.0-or-later" apps packages \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=dist \
  --exclude-dir=build \
  --exclude-dir=.svelte-kit >/dev/null

echo "== obvious secret sanity =="
if grep -R -l -E "(BEGIN (RSA|OPENSSH|PRIVATE) KEY|ghp_[A-Za-z0-9]|glpat-[A-Za-z0-9]|password *= *[^[:space:]'\"]+)" . \
  --exclude-dir=.git \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=dist \
  --exclude-dir=build \
  --exclude-dir=.svelte-kit \
  --exclude=verify-repo.sh; then
  echo "possible secret found"
  exit 1
fi

echo "ok"
