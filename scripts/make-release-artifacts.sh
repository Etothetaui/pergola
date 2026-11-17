#!/usr/bin/env bash
# Build the release artifact for one architecture.
#
#   scripts/make-release-artifacts.sh <version> <x86_64|aarch64> [binary] [out-dir]
#
# CI calls this once per target after `cargo build --release --target …`; run by
# hand it packages the host build. The contents are exactly what the
# `pergola-bin` package installs, so the artifact can be checked without a
# release:
#
#   scripts/make-release-artifacts.sh 1.0.0 x86_64 target/release/pergola /tmp/dist
#   bsdtar -tf /tmp/dist/pergola-1.0.0-x86_64-linux-gnu.tar.gz
#
# Writes <name>.tar.gz and <name>.tar.gz.sha256 into the out dir.
set -euo pipefail

usage() { sed -n '2,13p' "$0"; exit 2; }
[ $# -ge 2 ] || usage
VERSION="$1"
ARCH="$2"
BIN="${3:-target/release/pergola}"
OUT="${4:-dist}"

case "$ARCH" in
  x86_64) TRIPLE="x86_64-linux-gnu" ;;
  aarch64) TRIPLE="aarch64-linux-gnu" ;;
  *) echo "make-release-artifacts: arch must be x86_64 or aarch64, got '$ARCH'" >&2; exit 2 ;;
esac

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

[ -x "$BIN" ] || { echo "make-release-artifacts: no binary at $BIN (build it first)" >&2; exit 1; }
# The binary's own version has to agree with the artifact name, or the tarball
# lies about what it is. (The AUR generator also rejects a `-dev` version, so a
# mismatch is caught before anything is published.)
REPORTED="$("$BIN" --version 2>/dev/null | awk '{print $2}')"
[ "$REPORTED" = "$VERSION" ] || {
  echo "make-release-artifacts: binary reports '$REPORTED', expected '$VERSION'" >&2
  exit 1
}

NAME="pergola-$VERSION-$TRIPLE"
STAGE="$OUT/$NAME"
rm -rf "$STAGE"
mkdir -p "$STAGE/docs" "$STAGE/systemd" "$STAGE/autostart"

install -m 0755 "$BIN" "$STAGE/pergola"
install -m 0755 scripts/health.sh "$STAGE/pergola-health"
install -m 0644 LICENSE NOTICE README.md "$STAGE/"
install -m 0644 docs/SETUP.md docs/FIRST-RUN.md docs/UX-NOTES.md \
  docs/HISTORY-LESSONS.md "$STAGE/docs/"
install -m 0644 systemd/pergola.service "$STAGE/systemd/"
install -m 0644 autostart/pergola.desktop "$STAGE/autostart/"
# The config the daemon would create on first run, as an example: a package must
# not write into $HOME.
install -m 0644 packaging/aur/config.json "$STAGE/config.json"

# Deterministic archive: sorted names, fixed mtime, no gzip timestamp, so two
# runs over the same binaries produce the same bytes (and the same sha256).
tar --sort=name --mtime='@0' --owner=0 --group=0 --numeric-owner \
  -cf - -C "$OUT" "$NAME" | gzip -9n > "$OUT/$NAME.tar.gz"
( cd "$OUT" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256" )
rm -rf "$STAGE"

printf '%s\n' "$OUT/$NAME.tar.gz"
cat "$OUT/$NAME.tar.gz.sha256"
