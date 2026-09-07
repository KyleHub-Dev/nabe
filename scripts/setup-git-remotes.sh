#!/usr/bin/env sh
set -eu

GITHUB_URL="https://github.com/KyleHub-Dev/nabe.git"

if git remote get-url origin >/dev/null 2>&1; then
  git remote set-url origin "$GITHUB_URL"
else
  git remote add origin "$GITHUB_URL"
fi

# With no explicit push URL, pushes use the canonical fetch URL.
if git config --get-all remote.origin.pushurl >/dev/null; then
  git config --unset-all remote.origin.pushurl
fi

# Preserve tracking for branches that used the former convenience remote.
git config --get-regexp '^branch\..*\.remote$' | while read -r key remote; do
  if [ "$remote" = github ]; then
    git config "$key" origin
  fi
done
if git remote get-url github >/dev/null 2>&1; then
  git remote remove github
fi

git remote -v
