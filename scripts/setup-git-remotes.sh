#!/usr/bin/env sh
set -eu

CODEBERG_URL="ssh://git@codeberg.org/KyleHub/nabe.git"
GITHUB_URL="https://github.com/KyleHub-Dev/nabe.git"

if git remote get-url origin >/dev/null 2>&1; then
  git remote set-url origin "$CODEBERG_URL"
else
  git remote add origin "$CODEBERG_URL"
fi

git remote set-url --push origin "$CODEBERG_URL"
git remote set-url --add --push origin "$GITHUB_URL"

if git remote get-url github >/dev/null 2>&1; then
  git remote set-url github "$GITHUB_URL"
else
  git remote add github "$GITHUB_URL"
fi

git remote -v
