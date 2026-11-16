#!/usr/bin/env bash
# End-to-end test: a real X server (Xvfb), a real window manager, three real
# xterm windows, and the pergola daemon on top. Every assertion is made against
# geometry that the X server reports, not against the daemon's own bookkeeping
# alone.
#
#   scripts/e2e.sh                 uses target/release/pergola if present
#   scripts/e2e.sh --wm openbox    any of xfwm4 (default), openbox, i3
#   PERGOLA_BIN=... scripts/e2e.sh
#
# The WM is a parameter because the "any EWMH desktop" claim must be measured,
# not asserted: the 1.0.0 gate wants every command run under two window managers
# (docs/VERSIONING.md), and the summary line names the one that actually ran.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WM="${PERGOLA_TEST_WM:-xfwm4}"
while [ $# -gt 0 ]; do
  case "$1" in
    --wm) WM="${2:-}"; shift 2 ;;
    --wm=*) WM="${1#--wm=}"; shift ;;
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "e2e: unknown argument $1" >&2; exit 2 ;;
  esac
done
case "$WM" in
  xfwm4)   WM_CMD=(xfwm4 --compositor=off) ;;
  openbox) WM_CMD=(openbox) ;;
  i3)      WM_CMD=(i3 --no-config --shmlog-size 0) ;;
  *) echo "e2e: unknown window manager '$WM' (xfwm4|openbox|i3)" >&2; exit 2 ;;
esac
command -v "$WM" >/dev/null || { echo "e2e: $WM is not installed" >&2; exit 2; }
BIN="${PERGOLA_BIN:-$ROOT/target/release/pergola}"
[ -x "$BIN" ] || BIN="$ROOT/target/debug/pergola"
[ -x "$BIN" ] || { echo "build pergola first (cargo build)"; exit 1; }

DISP="${PERGOLA_TEST_DISPLAY:-:99}"
# A KEPT sandbox (PERGOLA_E2E_KEEP=1) or a stray daemon on the same display makes
# the next run fail in ways that look exactly like product bugs (measured
# 2026-11-15: half the geometry checks failed and blamed the engine, because a
# previous sandbox still owned :99). Refuse loudly instead.
x_alive() {
  if command -v xset >/dev/null; then xset -display "$DISP" q >/dev/null 2>&1; return $?; fi
  [ -S "/tmp/.X11-unix/X${DISP##*:}" ]
}
if [ -z "${PERGOLA_E2E_KEEP:-}" ] && x_alive; then
  echo "e2e: display $DISP is already in use (a KEPT sandbox?) — clean /tmp/pergola-e2e.*" >&2
  echo "     or run with PERGOLA_TEST_DISPLAY=:9x" >&2
  exit 2
fi
W=1600
H=900
TMP="$(mktemp -d /tmp/pergola-e2e.XXXXXX)"
SOCK="$TMP/sock"
export DISPLAY="$DISP"
export XDG_STATE_HOME="$TMP/state"
export XDG_CONFIG_HOME="$TMP/config"

PIDS=()
PASS=0
FAIL=0

say()  { printf '%s\n' "$*"; }
ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$*"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$*"; }
fail() { bad "$*"; exit 1; }

check_eq() { # expected actual label
  if [ "$1" = "$2" ]; then ok "$3 ($2)"; else bad "$3: expected [$1] got [$2]"; fi
}
check_near() { # expected actual tolerance label
  # An empty operand means the geometry lookup returned nothing; a bare `$(( ))`
  # on that prints "arithmetic syntax error" and hides which assertion failed.
  # Report the missing value as a FAIL instead.
  case "$1" in ''|*[!0-9-]*) bad "$4: expected $1±$3, got [${2:-<empty>}]"; return;; esac
  case "$2" in ''|*[!0-9-]*) bad "$4: expected $1±$3, got [<empty>]"; return;; esac
  local d=$(( $1 > $2 ? $1 - $2 : $2 - $1 ))
  if [ "$d" -le "$3" ]; then ok "$4 (expected $1, got $2)"; else bad "$4: expected $1±$3, got $2"; fi
}
check_true() { # condition-cmd label
  if eval "$1" >/dev/null 2>&1; then ok "$2"; else bad "$2"; fi
}

cleanup() {
  "$BIN" --socket "$SOCK" stop >/dev/null 2>&1 || true
  for pid in "${PIDS[@]:-}"; do kill "$pid" >/dev/null 2>&1 || true; done
  sleep 0.3
  for pid in "${PIDS[@]:-}"; do kill -9 "$pid" >/dev/null 2>&1 || true; done
  # PERGOLA_E2E_KEEP=1 keeps the sandbox (daemon.log, xvfb.log) for debugging.
  if [ -n "${PERGOLA_E2E_KEEP:-}" ]; then
    printf 'e2e sandbox kept: %s\n' "$TMP"
  else
    rm -rf "$TMP"
  fi
}
trap cleanup EXIT

# ---------------------------------------------------------------- sandbox ----
say "pergola e2e: $BIN"
say "sandbox: Xvfb $DISP ${W}x${H} + $WM + 3 xterms"
Xvfb "$DISP" -screen 0 "${W}x${H}x24" >"$TMP/xvfb.log" 2>&1 & PIDS+=($!)
sleep 1
"${WM_CMD[@]}" >"$TMP/wm.log" 2>&1 & PIDS+=($!)
sleep 2
XPIDS=()
for i in 1 2 3; do
  xterm -T "pergola-e2e-$i" -geometry 60x18+$((60*i))+$((40*i)) \
    -e sh -c 'exec sleep 3600' >/dev/null 2>&1 & XPIDS+=($!)
done
sleep 2

# ---------------------------------------------------------------- daemon -----
"$BIN" --socket "$SOCK" daemon >"$TMP/daemon.log" 2>&1 & PIDS+=($!)
for _ in $(seq 1 50); do "$BIN" --socket "$SOCK" status >/dev/null 2>&1 && break; sleep 0.1; done
"$BIN" --socket "$SOCK" status >/dev/null 2>&1 || { cat "$TMP/daemon.log"; fail "daemon did not start"; }
P() { "$BIN" --socket "$SOCK" "$@"; }

# One daemon per display: a second one must refuse, not unlink the live socket.
if "$BIN" --socket "$SOCK" daemon >"$TMP/daemon2.log" 2>&1; then
  bad "a second daemon is refused"
else
  ok "a second daemon is refused"
fi
P status >/dev/null 2>&1 || fail "the second daemon stole the live socket"

mapfile -t XWINS < <(xdotool search --class XTerm 2>/dev/null)
[ "${#XWINS[@]}" -ge 3 ] || fail "test windows did not appear"
W1="$(printf '0x%08x' "${XWINS[0]}")"
W2="$(printf '0x%08x' "${XWINS[1]}")"
W3="$(printf '0x%08x' "${XWINS[2]}")"
[ "$W1" != "$W2" ] && [ "$W2" != "$W3" ] || fail "test windows are not distinct"
say "windows: $W1 $W2 $W3"

# frame geometry as the X server sees it: client position/size + frame extents
frame_geom() {
  local win="$1" pos size left right top bottom
  pos=$(xdotool getwindowgeometry "$win" | sed -n 's/.*Position: \([-0-9]*\),\([-0-9]*\).*/\1 \2/p')
  size=$(xdotool getwindowgeometry "$win" | sed -n 's/.*Geometry: \([0-9]*\)x\([0-9]*\).*/\1 \2/p')
  local e
  e=$(xprop -id "$win" _NET_FRAME_EXTENTS | sed -n 's/.*= \([0-9]*\), \([0-9]*\), \([0-9]*\), \([0-9]*\)/\1 \2 \3 \4/p')
  read -r left right top bottom <<<"${e:-0 0 0 0}"
  read -r px py <<<"$pos"
  read -r sw sh <<<"$size"
  echo "$((px-left)) $((py-top)) $((sw+left+right)) $((sh+top+bottom))"
}

# Compare every docked panel against `pergola rects`, which is the same engine
# output the apply path uses. Tolerance covers WM decorations rounding.
check_geometry() {
  local label="$1" checked=0 skipped=0 id x y w h
  while read -r id x y w h; do
    [ -z "${id:-}" ] && continue
    # Windows outside the framed view are parked off-screen; some WMs clamp
    # them to keep a sliver reachable, so only on-screen tiles are asserted.
    if [ "$x" -lt 0 ] || [ "$y" -lt 0 ] || [ $((x+w)) -gt "$W" ] || [ $((y+h)) -gt "$H" ]; then
      skipped=$((skipped+1)); continue
    fi
    local win="0x${id#p-}" fg
    fg=$(frame_geom "$win")
    local ax ay aw ah; read -r ax ay aw ah <<<"$fg"
    # A client with a character grid (every terminal) snaps its size to
    # WM_NORMAL_HINTS resize increments, so the honest tolerance for a tile's
    # edges is its own increment, never less than 2 px. Compare edges rather
    # than origin+size: a snapped client can be one increment smaller while the
    # WM anchors it at the requested position, so the frame origin is exact but
    # the extent is not — and a real drift (a maximised window) is off by a
    # whole tile, far beyond one increment.
    local inc iw ih
    inc=$(xprop -id "$win" WM_NORMAL_HINTS 2>/dev/null \
      | sed -n 's/.*resize increment: \([0-9]*\) by \([0-9]*\).*/\1 \2/p')
    read -r iw ih <<<"${inc:-0 0}"
    [ "${iw:-0}" -gt 2 ] || iw=2
    [ "${ih:-0}" -gt 2 ] || ih=2
    local worst=0 d
    for d in $((ax-x)) $((ay-y)) $(((ax+aw)-(x+w))) $(((ay+ah)-(y+h))); do
      [ "${d#-}" -gt "$worst" ] && worst="${d#-}"
    done
    local tol=$iw; [ "$ih" -gt "$tol" ] && tol=$ih
    if [ "$worst" -le "$tol" ]; then
      ok "$label $id geometry ($x,$y ${w}x$h)"
    else
      bad "$label $id geometry: expected $x,$y ${w}x$h, got $ax,$ay ${aw}x$ah (edge drift $worst > $tol)"
    fi
    checked=$((checked+1))
  done < <(P rects)
  [ "$checked" -gt 0 ] || fail "$label: no panels to check"
  [ "$skipped" -eq 0 ] || say "       ($skipped panel(s) parked outside the framed view)"
}

json_field() { P tree | python3 -c "import json,sys; d=json.load(sys.stdin); print($1)"; }

# ------------------------------------------------------------------ tests ----
say
say "1. dock three windows"
xdotool windowactivate "$W1" >/dev/null 2>&1; sleep 0.4
P dock left >/dev/null || fail "dock left"
xdotool windowactivate "$W2" >/dev/null 2>&1; sleep 0.4
P dock right >/dev/null || fail "dock right"
xdotool windowactivate "$W3" >/dev/null 2>&1; sleep 0.4
P dock down >/dev/null || fail "dock down"
sleep 0.4
PANELS=$(P list | grep -c '^docked')
check_eq "3" "$PANELS" "panels docked"
check_geometry "after dock"

say
say "2. geometry covers the screen with gaps, no overlaps"
P rects > "$TMP/rects.txt"
python3 - "$W" "$H" "$TMP/rects.txt" <<'PY'
import sys
w,h=int(sys.argv[1]),int(sys.argv[2])
rows=[l.split() for l in open(sys.argv[3]) if l.strip()]
rects=[(r[0],int(r[1]),int(r[2]),int(r[3]),int(r[4])) for r in rows]
bad=0
for i,(a,ax,ay,aw,ah) in enumerate(rects):
    if ax<0 or ay<0 or ax+aw>w or ay+ah>h: print(f"  FAIL {a} outside screen"); bad+=1
    for b,bx,by,bw,bh in rects[i+1:]:
        if ax<bx+bw and bx<ax+aw and ay<by+bh and by<ay+ah:
            print(f"  FAIL {a} overlaps {b}"); bad+=1
if bad: sys.exit(1)
print(f"  ok   {len(rects)} tiles inside {w}x{h} with no overlaps")
PY
[ $? -eq 0 ] && PASS=$((PASS+1)) || FAIL=$((FAIL+1))

say
say "3. zoom"
P float --window "$W3" >/dev/null || fail "float W3"
sleep 0.3
FLOAT_BEFORE=$(P list | awk '$1=="floating"{print $2, $3, $4}' | head -1)
[ -n "$FLOAT_BEFORE" ] || fail "no floating window to check"
P everything >/dev/null
xdotool windowactivate "$W2" >/dev/null 2>&1; sleep 0.4
P frame-window >/dev/null || fail "frame-window"
sleep 0.4
read -r fx fy fw fh <<<"$(P rects | awk -v p="p-$(printf '%x' "$W2")" '$1==p {print $2,$3,$4,$5}')"
check_near "0" "$fx" 2 "framed window x"
check_near "0" "$fy" 2 "framed window y"
check_near "$W" "$fw" 2 "framed window width"
check_near "$H" "$fh" 2 "framed window height"
# The floating window must come back to the size it had before the zoom: live
# rects seen while zoomed are screen-space and must not be adopted as world
# positions.
P zoom-out >/dev/null || fail "zoom-out"
sleep 0.4
FLOAT_AFTER=$(P list | awk '$1=="floating"{print $2, $3, $4}' | head -1)
check_eq "$FLOAT_BEFORE" "$FLOAT_AFTER" "floating window geometry survives a zoom"
check_geometry "after zoom-out"

say
say "4. saved views and history"
P view 2 >/dev/null 2>&1 && true
P save-view 5 >/dev/null || fail "save-view"
P everything >/dev/null
P back >/dev/null || fail "back"
P forward >/dev/null || fail "forward"
P view 5 >/dev/null || fail "view"
FRAMED=$(json_field 'd["camera"]["framed"]')
[ -n "$FRAMED" ] && ok "camera restored to $FRAMED" || bad "camera not restored"

say
say "5. arrange / gap / float-all"
P arrange >/dev/null || fail "arrange"
sleep 0.3
check_geometry "after arrange"
DEPTH=$(json_field 'd["doc"]["root"].get("kind") == "split" and len(d["doc"]["root"]["children"]) or 0')
[ "$DEPTH" -ge 2 ] && ok "arranged tree has $DEPTH top-level children" || bad "arrange produced depth $DEPTH"
P gap 24 >/dev/null || fail "gap"
sleep 0.3
check_geometry "after gap 24"
# A ratio rebalances the split that holds the focused window; the reply shows
# the resulting weights so the change is observable without a UI.
P float-all >/dev/null; sleep 0.4
P arrange >/dev/null; sleep 0.4
xdotool windowactivate "$W1" >/dev/null 2>&1; sleep 0.3
RATIO_OUT=$(P ratio 0.3 2>&1)
case "$RATIO_OUT" in
  *"ratio 0.30"*) ok "ratio rebalances the focused split" ;;
  *) bad "ratio did not report a change: $RATIO_OUT" ;;
esac
# A ratio outside 0..1 is refused before it touches the tree.
if P ratio 1.5 >/dev/null 2>&1; then
  bad "ratio accepted 1.5"
else
  ok "ratio refuses a value outside 0..1"
fi
check_geometry "after ratio 0.3"
P float-all >/dev/null || fail "float-all"
sleep 0.3
check_eq "desktop" "$(json_field 'd["doc"]["root"]["id"]')" "float-all returns to the desktop leaf"
check_eq "3" "$(P list | grep -c '^floating')" "three floating windows"

say
say "6. persistence across a daemon restart"
P arrange >/dev/null; sleep 0.2
BEFORE=$(json_field 'json.dumps(d["doc"]["root"], sort_keys=True)')
P stop >/dev/null 2>&1
sleep 0.5
"$BIN" --socket "$SOCK" daemon >>"$TMP/daemon.log" 2>&1 & PIDS+=($!)
for _ in $(seq 1 50); do P status >/dev/null 2>&1 && break; sleep 0.1; done
AFTER=$(json_field 'json.dumps(d["doc"]["root"], sort_keys=True)')
check_eq "$BEFORE" "$AFTER" "tree survived restart"
sleep 0.3
check_geometry "after restart"

say
say "7. overlay and hotkeys"
P overlay on >/dev/null || fail "overlay on"
sleep 0.3
check_eq "on" "$(P status | sed -n 's/.*overlay \(on\|off\).*/\1/p')" "overlay is on"
xdotool windowactivate "$W1" >/dev/null 2>&1; sleep 0.3
P everything >/dev/null
P frame-window >/dev/null; sleep 0.3
xdotool key ctrl+alt+space; sleep 0.5
FRAMED=$(json_field 'd["camera"]["framed"]')
ROOT=$(json_field 'd["doc"]["root"]["id"]')
check_eq "$ROOT" "$FRAMED" "Ctrl+Alt+Space frames everything"
P overlay off >/dev/null

say
say "8. keyboard and mouse tiling"
P float-all >/dev/null; sleep 0.4
# A maximised window ignores _NET_MOVERESIZE_WINDOW, so docking has to clear
# that state first or the tile silently keeps the full-screen geometry.
wmctrl -i -r "$W1" -b add,maximized_vert,maximized_horz >/dev/null 2>&1; sleep 0.6
xdotool windowactivate "$W1" >/dev/null 2>&1; sleep 0.4
xdotool key ctrl+alt+shift+Left; sleep 0.8
if xprop -id "$W1" _NET_WM_STATE | grep -q MAXIMIZED; then
  bad "docking a maximised window left it maximised"
else
  ok "docking a maximised window clears the state"
fi
check_eq "1" "$(P list | grep -c '^docked')" "the maximised window is docked"
P float-all >/dev/null; sleep 0.4
xdotool windowactivate "$W1" >/dev/null 2>&1; sleep 0.4
xdotool key ctrl+alt+shift+Left; sleep 0.5
check_eq "1" "$(P list | grep -c '^docked')" "Ctrl+Alt+Shift+Left docks the focused window"
xdotool windowactivate "$W2" >/dev/null 2>&1; sleep 0.4
xdotool key ctrl+alt+shift+Right; sleep 0.5
check_eq "2" "$(P list | grep -c '^docked')" "Ctrl+Alt+Shift+Right docks beside it"
read -r GX GW1 GW2 <<<"$(P rects | awk 'NR==1 {x=$2; w=$4} NR==2 {print x+w+($2-(x+w))/2, w, $4; exit}')"
[ -n "${GX:-}" ] || fail "no draggable gap between the two tiles"
check_near "$GW1" "$GW2" 2 "the two tiles split evenly before the drag"
xdotool mousemove "$GX" 400; sleep 0.2
xdotool mousedown 1; sleep 0.1
for x in $((GX+30)) $((GX+70)) $((GX+120)); do xdotool mousemove "$x" 400; sleep 0.05; done
xdotool mouseup 1; sleep 0.5
NEW_W1=$(P rects | awk 'NR==1 {print $4}')
if [ "$NEW_W1" -gt $((GW1+80)) ]; then ok "dragging the gap resizes the split ($GW1 -> $NEW_W1)"; else bad "gap drag did not resize ($GW1 -> $NEW_W1)"; fi

# Mouse tiling: *drag* a floating window to the left screen edge and let go.
# The gesture, not a geometry poke: wmctrl -e is interpreted per WM (openbox
# coalesces it into one ConfigureNotify where xfwm4 emits a stream, and it
# re-places the window at 0,0 — the top-left corner — which the daemon then
# reads as a real drag to the left edge and docks before the gesture starts).
# The daemon's edge detection exists for a real drag, so the test must be one,
# starting from wherever the window already floats.
P float --window "$W3" >/dev/null 2>&1; sleep 0.4
eval "$(xdotool getwindowgeometry --shell "$W3")"   # X, Y, WIDTH, HEIGHT
xdotool windowactivate "$W3" >/dev/null 2>&1; sleep 0.3
xdotool mousemove $((X + WIDTH / 2)) $((Y + 12)) sleep 0.15 \
  keydown alt sleep 0.1 mousedown 1 sleep 0.15
for x in 700 500 300 120 20 2; do xdotool mousemove "$x" $((Y + 12)); sleep 0.08; done
xdotool mouseup 1 sleep 0.1 keyup alt
sleep 1.5
if P list | grep -q "^docked.*$W3"; then ok "a window dragged to the left edge is docked"; else bad "edge drag did not dock $W3"; fi
# Pulling that tile back off the screen edge floats it where it was dropped.
for x in 320 400 480; do wmctrl -i -r "$W3" -e "0,$x,220,320,240" >/dev/null 2>&1; sleep 0.15; done
sleep 2.5
if P list | grep -q "^floating.*$W3"; then ok "a docked tile dragged away floats again"; else bad "drag-out did not float $W3"; fi
P arrange >/dev/null; sleep 0.4

# Holding a chord must act once. The X server auto-repeats a held key ~11
# times a second, so a bound command would race ("back" held for a second
# would walk eleven history entries). The daemon latches the chord on press
# and clears it on release, so a hold is one press.
chord_for() { # command -> xdotool key spec, from the bindings actually installed
  local label
  label=$(P keys | awk -v c="$1" '$2==c {print $1; exit}')
  [ -n "$label" ] || return 1
  # Lowercase only the modifier tokens: xdotool takes "ctrl" or "Ctrl" but
  # needs the key name exactly as X spells it. Lowercasing the whole label
  # turned "Page_Down" into "page_down", which xdotool rejects, so the chord
  # silently never fired.
  printf '%s' "$label" | awk -F+ '{
    n = NF; out = "";
    for (i = 1; i < n; i++) out = out tolower($i) "+";
    key = $n;
    if (key == "[") key = "bracketleft";
    if (key == "]") key = "bracketright";
    print out key;
  }'
}
build_history() {
  P everything >/dev/null; sleep 0.4
  for w in "$W1" "$W2" "$W3"; do
    xdotool windowactivate "$w" >/dev/null 2>&1; sleep 0.35
    P frame-window >/dev/null; sleep 0.35
  done
}
BACK=$(chord_for back) || fail "back is not bound"
build_history
PRE=$(json_field 'd["camera"]["framed"]')
xdotool key "$BACK"; sleep 0.5
ONE=$(json_field 'd["camera"]["framed"]')
build_history
xdotool keydown "$BACK"; sleep 1.2; xdotool keyup "$BACK"; sleep 0.6
HELD=$(json_field 'd["camera"]["framed"]')
# The hold must move the camera (not be dropped) and move it exactly as far as
# one deliberate tap — the server's ~11 repeats must not each step back.
if [ "$HELD" = "$PRE" ]; then bad "a held chord did nothing (still $PRE)"; else
  check_eq "$ONE" "$HELD" "holding a chord goes back once, not eleven times"
fi

say
say "9. config validation and error paths"
if "$BIN" --socket "$SOCK" dock sideways >/dev/null 2>&1; then bad "a bad edge is rejected"; else ok "a bad edge is rejected"; fi
if "$BIN" --socket "$SOCK" view 0 >/dev/null 2>&1; then bad "a bad view slot is rejected"; else ok "a bad view slot is rejected"; fi
if "$BIN" sim --json | python3 -c 'import json,sys; json.load(sys.stdin)' >/dev/null; then ok "sim --json is valid JSON"; else bad "sim --json is not valid JSON"; fi
if P tree | python3 -c 'import json,sys; d=json.load(sys.stdin); d["doc"]["root"]["kind"]' >/dev/null; then ok "tree is valid JSON"; else bad "tree is not valid JSON"; fi

# Config-driven keybindings: a `bindings` map overrides the default chords per
# command, and `reload` re-grabs without a daemon restart. Written last so the
# key presses above ran against the default plan.
mkdir -p "$XDG_CONFIG_HOME/pergola"
printf '%s' '{"bindings":{"view 3":["Super+9"],"overlay":[]}}' \
  > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1
sleep 0.4
if P keys | grep -c 'Super+9 .*view 3' >/dev/null; then
  ok "a config binding overrides a default chord and reload re-grabs"
else
  bad "config binding did not take effect: $(P keys | grep 'view 3' || echo 'not bound')"
fi
if P keys --check | grep -c '^off .*overlay' >/dev/null; then
  ok "an empty candidate list shows as a deliberate unbind"
else
  bad "an empty candidate list was not reported as off: $(P keys --check | grep overlay || echo 'not listed')"
fi

# `hotkeys: false` at reload must release the grabs so the desktop gets its
# chords back; `hotkeys: true` must take them again. Both without a restart.
printf '%s' '{"hotkeys":false}' > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1
sleep 0.3
if [ -z "$(P keys)" ]; then
  ok "hotkeys: false at reload releases every grab"
else
  bad "hotkeys: false did not release the grabs: $(P keys | head -3 | tr '\n' ' ')"
fi
printf '%s' '{"hotkeys":true}' > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1
sleep 0.3
if [ "$(P keys | grep -c 'everything')" -ge 1 ]; then
  ok "hotkeys: true at reload takes the grabs back"
else
  bad "hotkeys: true did not re-grab"
fi

# auto_dock is a tri-state. `off` (the default) leaves a new window floating;
# `new` docks a window that appears after the daemon started. This is the
# backlog's "no window is adopted into a space it was not opened in": off never
# adopts, and the three startup windows stay where they were.
printf '%s' '{"auto_dock":"off","hotkeys":true}' > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1; sleep 0.3
xterm -T pergola-e2e-autodock-off -geometry 60x18+320+320 -e sh -c 'exec sleep 3600' >/dev/null 2>&1 & AUTO_OFF_PID=$!; XPIDS+=($AUTO_OFF_PID)
# reconcile runs on a 2 s timer, so poll for the seat instead of guessing a sleep.
AUTO_OFF=""
for _ in $(seq 1 40); do
  AUTO_OFF=$(xdotool search --name pergola-e2e-autodock-off 2>/dev/null | head -1)
  [ -n "$AUTO_OFF" ] && P list | grep -c "^floating.*$(printf '0x%08x' "$AUTO_OFF")" >/dev/null && break
  sleep 0.1
done
if [ -n "$AUTO_OFF" ] && P list | grep -c "^floating.*$(printf '0x%08x' "$AUTO_OFF")" >/dev/null; then
  ok "auto_dock off leaves a new window floating"
else
  bad "auto_dock off adopted a new window"
fi
kill "$AUTO_OFF_PID" 2>/dev/null; sleep 0.4
printf '%s' '{"auto_dock":"new","hotkeys":true}' > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1; sleep 0.3
xterm -T pergola-e2e-autodock-new -geometry 60x18+340+340 -e sh -c 'exec sleep 3600' >/dev/null 2>&1 & AUTO_NEW_PID=$!; XPIDS+=($AUTO_NEW_PID)
AUTO_NEW=""
for _ in $(seq 1 40); do
  AUTO_NEW=$(xdotool search --name pergola-e2e-autodock-new 2>/dev/null | head -1)
  [ -n "$AUTO_NEW" ] && P list | grep -c "^docked.*$(printf '0x%08x' "$AUTO_NEW")" >/dev/null && break
  sleep 0.1
done
if [ -n "$AUTO_NEW" ] && P list | grep -c "^docked.*$(printf '0x%08x' "$AUTO_NEW")" >/dev/null; then
  ok "auto_dock new docks a window opened after startup"
else
  bad "auto_dock new did not dock a new window"
fi
kill "$AUTO_NEW_PID" 2>/dev/null; sleep 0.4
printf '%s' '{"hotkeys":true}' > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1

# The shell commands: `bind` and `unbind` write config.json and reload, so a
# user never has to edit JSON by hand. They must preserve other settings.
printf '%s' '{"gap":6.0,"hotkeys":true}' > "$XDG_CONFIG_HOME/pergola/config.json"
BIND_OUT=$("$BIN" --socket "$SOCK" bind "gap 12" Super+9 2>&1)
case "$BIND_OUT" in
  *"bound 'gap 12' to Super+9"*) ok "bind writes the chord and reloads" ;;
  *) bad "bind did not report the binding: $BIND_OUT" ;;
esac
if grep -q '"gap": *6.0' "$XDG_CONFIG_HOME/pergola/config.json"; then
  ok "bind preserves the other settings in config.json"
else
  bad "bind lost the other settings: $(cat "$XDG_CONFIG_HOME/pergola/config.json")"
fi
if [ "$(P keys | grep -c 'Super+9')" -ge 1 ]; then
  ok "a bound chord is live after reload"
else
  bad "bind did not take effect: $(P keys | head -3 | tr '\n' ' ')"
fi
UNBIND_OUT=$("$BIN" --socket "$SOCK" unbind "gap 12" 2>&1)
case "$UNBIND_OUT" in
  *"unbound 'gap 12'"*) ok "unbind clears the chord" ;;
  *) bad "unbind did not report: $UNBIND_OUT" ;;
esac
if [ "$(P keys --check | grep -c '^off .*gap 12')" -ge 1 ]; then
  ok "an unbound command shows as off"
else
  bad "unbind did not show as off: $(P keys --check | grep 'gap 12' || echo 'not listed')"
fi
if "$BIN" --socket "$SOCK" bind "not-a-command" Super+9 >/dev/null 2>&1; then
  bad "bind accepts an unknown command"
else
  ok "bind rejects an unknown command"
fi
if "$BIN" --socket "$SOCK" bind "gap 12" NotAChord >/dev/null 2>&1; then
  bad "bind accepts an unparseable chord"
else
  ok "bind rejects an unparseable chord"
fi
DEFAULT_OUT=$("$BIN" --socket "$SOCK" bind "gap 12" --default 2>&1)
case "$DEFAULT_OUT" in
  *"restored the default chords"*) ok "bind --default removes the override" ;;
  *) bad "bind --default did not report: $DEFAULT_OUT" ;;
esac
if grep -q '"bindings"' "$XDG_CONFIG_HOME/pergola/config.json"; then
  bad "bind --default left an empty bindings map behind"
else
  ok "bind --default leaves a clean config"
fi
# A malformed config is never silently regenerated: the user's typo stays.
printf '%s' '{"gap": 6.0, "bindings": {"back": ["Super+b"}' > "$XDG_CONFIG_HOME/pergola/config.json"
if "$BIN" --socket "$SOCK" bind "back" Super+z >/dev/null 2>&1; then
  bad "bind rewrote a malformed config"
else
  ok "bind refuses to rewrite a malformed config"
fi
if grep -q 'Super+b' "$XDG_CONFIG_HOME/pergola/config.json"; then
  ok "the malformed config is left untouched"
else
  bad "the malformed config was overwritten"
fi

say
say "10. tab groups"
# A tab group is one panel holding several windows: every view is mapped at the
# panel rectangle and only the selected one is raised. The document model has
# always supported this; before 0.4.0-dev nothing could create, switch or
# repair a group, so a closed tab took the whole panel with it.
printf '%s' '{"gap":6.0,"hotkeys":true}' > "$XDG_CONFIG_HOME/pergola/config.json"
"$BIN" --socket "$SOCK" reload >/dev/null 2>&1
sleep 0.3
P float-all >/dev/null; sleep 0.4
P arrange >/dev/null; sleep 0.4

# The first panel with more than one view, as "id count selected".
tab_panel() {
  P tree | python3 -c '
import json, sys
d = json.load(sys.stdin)
def walk(n):
    if n.get("kind") == "panel":
        yield n
    for c in n.get("children", []):
        yield from walk(c)
for p in walk(d["doc"]["root"]):
    if len(p["views"]) > 1:
        print(p["id"], len(p["views"]), p.get("selected") or "-")
        break
'
}

# Fold W2 into W1's panel: `tab` with no --target joins the panel that had
# focus before the tabbed window.
xdotool windowactivate "$W1" >/dev/null 2>&1; sleep 0.4
TAB_OUT=$("$BIN" --socket "$SOCK" tab --window "$W2" 2>&1)
case "$TAB_OUT" in
  *"tabbed"*) ok "tab folds a window into a panel" ;;
  *) bad "tab did not report the fold: $TAB_OUT" ;;
esac
sleep 0.5
S_W1=$(printf 's-%x' "$((W1))")
S_W2=$(printf 's-%x' "$((W2))")
read -r TAB_ID TAB_COUNT TAB_SEL <<EOF
$(tab_panel)
EOF
check_eq "2" "${TAB_COUNT:-0}" "the panel now holds both windows"
check_eq "$S_W2" "${TAB_SEL:-}" "the tabbed window is the shown view"

# Both views are mapped at the panel rectangle: the X server must agree.
eval "$(xdotool getwindowgeometry --shell "$W1")"; read -r A_X A_Y A_W A_H <<<"$X $Y $WIDTH $HEIGHT"
eval "$(xdotool getwindowgeometry --shell "$W2")"; read -r B_X B_Y B_W B_H <<<"$X $Y $WIDTH $HEIGHT"
check_near "$A_X" "$B_X" 2 "both tabs sit at the same x"
check_near "$A_Y" "$B_Y" 2 "both tabs sit at the same y"
check_near "$A_W" "$B_W" 2 "both tabs have the same width"
check_near "$A_H" "$B_H" 2 "both tabs have the same height"

# Exactly one is raised: in _NET_CLIENT_LIST_STACKING (bottom to top) the
# shown view must come after the hidden one.
stack_rank() { # window -> 1-based position in the stacking order (bottom = 1)
  # Compare numeric ids: the WM prints 0x60000c, the test has 0x0060000c.
  xprop -root _NET_CLIENT_LIST_STACKING 2>/dev/null | python3 -c '
import sys
want = int(sys.argv[1], 16)
tail = sys.stdin.read().split("=", 1)[-1]
ids = [int(t, 16) for t in tail.replace(",", " ").split() if t.startswith("0x")]
print(ids.index(want) + 1 if want in ids else "")
' "$1"
}
R_W1=$(stack_rank "$W1"); R_W2=$(stack_rank "$W2")
if [ -n "$R_W1" ] && [ -n "$R_W2" ] && [ "$R_W2" -gt "$R_W1" ]; then
  ok "only the shown tab is raised ($W2 above $W1)"
else
  bad "the shown tab is not the top one: ranks W1=${R_W1:-?} W2=${R_W2:-?}"
fi

# Cycling moves the shown view and raises the other one.
P tab-next >/dev/null || fail "tab-next"
sleep 0.5
read -r _ _ TAB_SEL <<EOF
$(tab_panel)
EOF
check_eq "$S_W1" "${TAB_SEL:-}" "tab-next shows the other view"
R_W1=$(stack_rank "$W1"); R_W2=$(stack_rank "$W2")
if [ -n "$R_W1" ] && [ -n "$R_W2" ] && [ "$R_W1" -gt "$R_W2" ]; then
  ok "tab-next raises the newly shown view"
else
  bad "tab-next did not raise the shown view: ranks W1=${R_W1:-?} W2=${R_W2:-?}"
fi
P tab-prev >/dev/null || fail "tab-prev"
sleep 0.4
read -r _ _ TAB_SEL <<EOF
$(tab_panel)
EOF
check_eq "$S_W2" "${TAB_SEL:-}" "tab-prev returns to the previous view"

# The default chord must do the same thing. Cycling by key is the whole point:
# a tab group the user cannot switch from the keyboard is not a feature.
if P keys --check | grep -c 'tab-next' >/dev/null; then
  NEXT_CHORD=$(chord_for tab-next) || fail "tab-next has no chord"
  xdotool key "$NEXT_CHORD"; sleep 0.5
  read -r _ _ TAB_SEL <<EOF
$(tab_panel)
EOF
  check_eq "$S_W1" "${TAB_SEL:-}" "the tab-next chord shows the other view"
  xdotool key "$(chord_for tab-prev)"; sleep 0.4
  read -r _ _ TAB_SEL <<EOF
$(tab_panel)
EOF
  check_eq "$S_W2" "${TAB_SEL:-}" "the tab-prev chord returns to the first view"
else
  bad "tab-next is not bound by default"
fi

# Closing the shown tab must not collapse the panel: the sibling stays put.
# The daemon reconciles every 2 s, so give the destroy time to land.
P tab-select "$S_W1" >/dev/null || fail "tab-select"
sleep 0.4
read -r _ _ TAB_SEL <<EOF
$(tab_panel)
EOF
check_eq "$S_W1" "${TAB_SEL:-}" "tab-select shows a named view"
xdotool windowkill "$W1" >/dev/null 2>&1
sleep 2.6
if P tree | grep -c "$S_W2" >/dev/null; then
  ok "closing one tab leaves its sibling in the layout"
else
  bad "closing one tab removed the whole panel"
fi
if P tree | grep -c "$S_W1" >/dev/null; then
  bad "the closed tab is still in the layout"
else
  ok "the closed tab is dropped from the layout"
fi
if P list | grep -c "^docked.*$W2" >/dev/null; then
  ok "the surviving tab is still docked and mapped"
else
  bad "the surviving tab was lost: $(P list | tr '\n' ' ')"
fi

# A floating window can be folded into a panel as a tab too: `tab` accepts a
# seat off the desktop layer and docks it into the target. W3 is a separate
# panel after the group above collapsed, so float it and tab it into W2's.
if P list | grep -c "^docked.*$W3" >/dev/null; then
  P float --window "$W3" >/dev/null 2>&1 || fail "float W3"
  sleep 0.5
  P tab --window "$W3" >/dev/null 2>&1 || fail "tab a floating window"
  sleep 0.5
  read -r _ TAB_COUNT TAB_SEL <<EOF
$(tab_panel)
EOF
  check_eq "2" "${TAB_COUNT:-0}" "a floating window folds into a tab group"
  S_W3=$(printf 's-%x' "$((W3))")
  check_eq "$S_W3" "${TAB_SEL:-}" "the folded floating window is the shown view"
  if P list | grep -c "^docked.*$W3" >/dev/null; then
    ok "the folded window is docked, not floating"
  else
    bad "the folded window is still floating: $(P list | tr '\n' ' ')"
  fi
else
  bad "W3 was not a docked panel to float"
fi

say
say "11. screenshot"
import -window root "$TMP/shot.png" 2>/dev/null && cp "$TMP/shot.png" /tmp/pergola-e2e-shot.png && ok "screenshot at /tmp/pergola-e2e-shot.png"

say
printf 'e2e: %d passed, %d failed (wm=%s)\n' "$PASS" "$FAIL" "$WM"
[ "$FAIL" -eq 0 ]
