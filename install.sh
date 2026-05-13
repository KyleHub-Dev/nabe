#!/usr/bin/env bash
set -euo pipefail

SOURCE_URL="${NABE_SOURCE_URL:-https://codeberg.org/KyleHub/nabe/archive/main.tar.gz}"
INSTALL_PATH="${NABE_INSTALL_PATH:-/usr/local/bin/nabe}"

need() {
  command -v "$1" >/dev/null 2>&1
}

sudo_cmd() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  else
    sudo "$@"
  fi
}

if ! need curl || ! need tar || ! need gzip || ! need go; then
  if need apt-get; then
    sudo_cmd apt-get update
    sudo_cmd env DEBIAN_FRONTEND=noninteractive apt-get install -y ca-certificates curl tar gzip golang-go
  else
    echo "install.sh requires curl, tar, gzip, and go; automatic dependency install currently supports apt-get only" >&2
    exit 1
  fi
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fsSL "$SOURCE_URL" -o "$tmp/nabe.tar.gz"
tar -xzf "$tmp/nabe.tar.gz" -C "$tmp"

src="$(find "$tmp" -path '*/apps/cli/go.mod' -print -quit)"
if [ -z "$src" ]; then
  echo "downloaded Nabe source did not contain apps/cli/go.mod" >&2
  exit 1
fi

cli_dir="$(dirname "$src")"
bin="$tmp/nabe"
(cd "$cli_dir" && go build -o "$bin" .)
install_dir="$(dirname "$INSTALL_PATH")"
if [ -w "$install_dir" ]; then
  install -m 0755 "$bin" "$INSTALL_PATH"
else
  sudo_cmd install -m 0755 "$bin" "$INSTALL_PATH"
fi

"$INSTALL_PATH" version
