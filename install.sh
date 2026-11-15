#!/usr/bin/env bash
# Install pergola for the current user: binary, config, autostart and a systemd
# user unit. Nothing here needs root, and nothing replaces the desktop's own
# window manager.
#
#   ./install.sh              build, install, start
#   ./install.sh --no-start   install without starting the daemon
#   PREFIX=/usr/local ./install.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BINDIR="${BINDIR:-$PREFIX/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/pergola"
STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/pergola"
START=1
for arg in "$@"; do
  case "$arg" in
    --no-start) START=0 ;;
    *) echo "unknown option: $arg" >&2; exit 1 ;;
  esac
done

command -v cargo >/dev/null || { echo "cargo is required (install rust)"; exit 1; }
echo "building release…"
( cd "$ROOT" && cargo build --release )

echo "installing to $BINDIR/pergola"
mkdir -p "$BINDIR" "$CONFIG_DIR" "$STATE_DIR"
install -m 0755 "$ROOT/target/release/pergola" "$BINDIR/pergola"
install -m 0755 "$ROOT/scripts/health.sh" "$BINDIR/pergola-health"

if [ ! -f "$CONFIG_DIR/config.json" ]; then
  cat >"$CONFIG_DIR/config.json" <<'JSON'
{
  "gap": 6.0,
  "minimum_share": 0.05,
  "auto_dock": false,
  "overlay": false,
  "desktop_share": 0.0,
  "terminal": "xfce4-terminal",
  "hotkeys": true
}
JSON
  echo "wrote $CONFIG_DIR/config.json"
fi

# systemd user unit (used by desktops that start graphical-session.target)
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p "$UNIT_DIR"
sed "s|%h/.local/bin/pergola|$BINDIR/pergola|" \
  "$ROOT/systemd/pergola.service" >"$UNIT_DIR/pergola.service"
if command -v systemctl >/dev/null; then
  systemctl --user daemon-reload 2>/dev/null || true
  systemctl --user enable pergola.service 2>/dev/null || true
fi

# XDG autostart entry: what XFCE, GNOME on X11, KDE, LXQt, MATE use
AUTOSTART="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
mkdir -p "$AUTOSTART"
sed "s|%h/.local/bin/pergola|$BINDIR/pergola|" \
  "$ROOT/autostart/pergola.desktop" >"$AUTOSTART/pergola.desktop"
echo "autostart entry: $AUTOSTART/pergola.desktop"

if [ "$START" -eq 1 ]; then
  if "$BINDIR/pergola" status >/dev/null 2>&1; then
    echo "daemon already running"
  else
    echo "starting the daemon…"
    setsid "$BINDIR/pergola" daemon >/dev/null 2>&1 &
    sleep 1
    "$BINDIR/pergola" status || true
  fi
fi

echo
echo "installed. try:"
echo "  pergola status"
echo "  pergola keys --check"
echo "  pergola-health        # soak check: daemon, cpu, layout, drift, orphans"
echo "  pergola dock left     # dock the focused window"
echo "  pergola arrange       # tile every window into a fractal tree"
