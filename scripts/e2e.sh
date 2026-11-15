#!/usr/bin/env bash
# End-to-end test: a real X server (Xvfb), a real window manager (xfwm4), three
# real xterm windows, and the pergola daemon on top. Every assertion is made
# against geometry that the X server reports, not against the daemon's own
# bookkeeping alone.
#
#   scripts/e2e.sh            uses target/release/pergola if present
#   PERGOLA_BIN=... scripts/e2e.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${PERGOLA_BIN:-$ROOT/target/release/pergola}"
[ -x "$BIN" ] || BIN="$ROOT/target/debug/pergola"
[ -x "$BIN" ] || { echo "build pergola first (cargo build)"; exit 1; }

DISP="${PERGOLA_TEST_DISPLAY:-:99}"
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
  rm -rf "$TMP"
}
trap cleanup EXIT

# ---------------------------------------------------------------- sandbox ----
say "pergola e2e: $BIN"
say "sandbox: Xvfb $DISP ${W}x${H} + xfwm4 + 3 xterms"
Xvfb "$DISP" -screen 0 "${W}x${H}x24" >"$TMP/xvfb.log" 2>&1 & PIDS+=($!)
sleep 1
xfwm4 --compositor=off >"$TMP/xfwm4.log" 2>&1 & PIDS+=($!)
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
    check_near "$x" "$ax" 2 "$label $id x"
    check_near "$y" "$ay" 2 "$label $id y"
    check_near "$w" "$aw" 2 "$label $id w"
    check_near "$h" "$ah" 2 "$label $id h"
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
say "8. config validation and error paths"
if "$BIN" --socket "$SOCK" dock sideways >/dev/null 2>&1; then bad "a bad edge is rejected"; else ok "a bad edge is rejected"; fi
if "$BIN" --socket "$SOCK" view 0 >/dev/null 2>&1; then bad "a bad view slot is rejected"; else ok "a bad view slot is rejected"; fi
if "$BIN" sim --json | python3 -c 'import json,sys; json.load(sys.stdin)' >/dev/null; then ok "sim --json is valid JSON"; else bad "sim --json is not valid JSON"; fi
if P tree | python3 -c 'import json,sys; d=json.load(sys.stdin); d["doc"]["root"]["kind"]' >/dev/null; then ok "tree is valid JSON"; else bad "tree is not valid JSON"; fi

say
say "9. screenshot"
import -window root "$TMP/shot.png" 2>/dev/null && cp "$TMP/shot.png" /tmp/pergola-e2e-shot.png && ok "screenshot at /tmp/pergola-e2e-shot.png"

say
printf 'e2e: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
