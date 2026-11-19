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
  "hotkeys": true,
  "edge_dock": true
}
JSON
  echo "wrote $CONFIG_DIR/config.json"
fi

# systemd user unit (used by desktops that start graphical-session.target)
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p "$UNIT_DIR"
# The unit cannot inherit DISPLAY from the desktop session, so bake the display
# this install runs on, not a hard-coded :0 (re-run install.sh after changing it).
sed -e "s|%h/.local/bin/pergola|$BINDIR/pergola|" \
    -e "s|Environment=DISPLAY=:0|Environment=DISPLAY=${DISPLAY:-:0}|" \
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

# A normal launcher entry makes the existing action palette findable through
# the desktop's application menu. It does not start another daemon.
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$APPS"
sed "s|%h/.local/bin/pergola|$BINDIR/pergola|g" \
  "$ROOT/desktop/pergola-actions.desktop" >"$APPS/pergola-actions.desktop"
echo "action launcher: $APPS/pergola-actions.desktop"

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
NEXT_BIN="$BINDIR/pergola"
NEXT_HEALTH="$BINDIR/pergola-health"
if [ "$(command -v pergola || true)" != "$BINDIR/pergola" ]; then
  echo "  note: your shell does not resolve 'pergola' to this install; the full commands below use the new binary"
fi
echo "  $NEXT_BIN status"
echo "  open 'Pergola Actions' from your desktop's application menu"
echo "  $NEXT_BIN keys --check"
echo "  $NEXT_HEALTH        # soak check: daemon, cpu, layout, drift, orphans"
echo "  $NEXT_BIN dock left  # dock the focused window"
echo "  $NEXT_BIN arrange    # tile every window into a fractal tree"
if [ -n "${PERGOLA_MENU:-}" ]; then
  if ! command -v "$PERGOLA_MENU" >/dev/null && [ ! -x "$PERGOLA_MENU" ]; then
    echo "  note: PERGOLA_MENU does not name an executable menu; fix or unset it"
  fi
elif ! command -v rofi >/dev/null && ! command -v zenity >/dev/null &&
     ! command -v yad >/dev/null && ! command -v dmenu >/dev/null; then
  echo "  note: the action menu needs rofi, zenity, yad or dmenu; install one or set PERGOLA_MENU"
fi
