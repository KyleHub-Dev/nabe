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
  apps/web/package.json \
  apps/api/package.json \
  apps/worker/package.json \
  apps/speiche/go.mod \
  packages/edge-protocol/LICENSE \
  packages/adguard-client/LICENSE \
  packages/db/LICENSE \
  deploy/nabe/compose.yaml \
  deploy/speiche/compose.yaml; do
  test -e "$path" || { echo "missing $path"; exit 1; }
done

echo "== package license sanity =="
grep -R "Apache-2.0\\|AGPL-3.0-or-later" apps packages >/dev/null

echo "== obvious secret sanity =="
if grep -R -n -E "(BEGIN (RSA|OPENSSH|PRIVATE) KEY|ghp_[A-Za-z0-9]|glpat-[A-Za-z0-9]|password *= *[^[:space:]'\"]+)" . \
  --exclude-dir=.git \
  --exclude-dir=node_modules \
  --exclude-dir=dist \
  --exclude-dir=build \
  --exclude-dir=.svelte-kit \
  --exclude=verify-repo.sh; then
  echo "possible secret found"
  exit 1
fi

echo "ok"
