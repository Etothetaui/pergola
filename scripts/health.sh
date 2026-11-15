#!/usr/bin/env bash
# Pergola health check: the soak test in one command.
#
#   scripts/health.sh              check the running daemon and its layout
#   scripts/health.sh --e2e        also run the full regression suite (~3 min)
#   scripts/health.sh --quiet      print nothing when healthy (exit code still tells)
#   scripts/health.sh --json       one machine-readable line on stdout
#
# Checks, in order: daemon answers, CPU under load, layout invariants, geometry
# drift against what the X server reports, docked windows that no longer exist.
# Exits 1 on any failure, so a timer or a scheduler can act on it.
set -uo pipefail

# When installed as `pergola-health` the source tree is not next to the script;
# find it by walking up, then by PERGOLA_REPO, then the usual checkout path.
find_repo() {
  local d
  d="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" 2>/dev/null && pwd)"
  while [ -n "$d" ] && [ "$d" != "/" ]; do
    [ -x "$d/scripts/e2e.sh" ] && { printf '%s' "$d"; return 0; }
    d="$(dirname "$d")"
  done
  for cand in "${PERGOLA_REPO:-}" "$HOME/Projects/pergola"; do
    [ -n "$cand" ] && [ -x "$cand/scripts/e2e.sh" ] && { printf '%s' "$cand"; return 0; }
  done
  return 1
}
ROOT="$(find_repo || printf '%s' "$(pwd)")"
BIN="${PERGOLA_BIN:-$(command -v pergola || echo "$ROOT/target/release/pergola")}"
LOG="${PERGOLA_HEALTH_LOG:-${XDG_STATE_HOME:-$HOME/.local/state}/pergola/health.jsonl}"
E2E=0
QUIET=0
JSON=0
for arg in "$@"; do
  case "$arg" in
    --e2e) E2E=1 ;;
    --quiet) QUIET=1 ;;
    --json) JSON=1 ;;
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "health: unknown option $arg" >&2; exit 2 ;;
  esac
done

FAILURES=()
CHECKS=()
note() { [ "$QUIET" -eq 1 ] || printf '%s\n' "$*"; }
pass() { CHECKS+=("$1"); }
fail() { FAILURES+=("$1"); CHECKS+=("FAIL $1"); note "  FAIL $1"; }
check() { # name, command
  if eval "$2" >/dev/null 2>&1; then pass "$1"; note "  ok   $1"; else fail "$1"; fi
}

[ -x "$BIN" ] || { echo "health: no pergola binary (set PERGOLA_BIN)" >&2; exit 1; }
note "pergola health: $BIN"

# 1. the daemon answers ------------------------------------------------------
STATUS="$(timeout 5 "$BIN" status 2>&1)"
if [ $? -ne 0 ] || [ -z "$STATUS" ]; then
  fail "daemon does not answer: $STATUS"
  STATUS=""
else
  pass "daemon answers"
  note "  ok   daemon answers"
fi

# 2. CPU ---------------------------------------------------------------------
PID="$(pgrep -x pergola | head -1)"
CPU="0.0"
if [ -n "${PID:-}" ]; then
  read -r utime1 stime1 <<<"$(awk '{print $14, $15}' /proc/"$PID"/stat)"
  sleep 2
  read -r utime2 stime2 <<<"$(awk '{print $14, $15}' /proc/"$PID"/stat)"
  CPU=$(python3 -c "print(round((($utime2+$stime2)-($utime1+$stime1))/2.0,1))")
  if python3 -c "import sys; sys.exit(0 if $CPU < 10.0 else 1)"; then
    pass "cpu ${CPU}%"
    note "  ok   cpu ${CPU}% of one core"
  else
    fail "cpu ${CPU}% of one core (>10%)"
  fi
else
  fail "no pergola daemon process"
fi

# 3. layout invariants -------------------------------------------------------
TREE="$("$BIN" tree 2>/dev/null)"
INVARIANTS="$(printf '%s' "$TREE" | python3 -c '
import json, sys
problems, ids, desktops = [], [], 0
try:
    doc = json.load(sys.stdin)
except Exception as exc:
    print("unreadable layout json: %s" % exc); raise SystemExit(0)
root = doc.get("doc", {}).get("root")
def walk(n):
    global desktops
    nid = n.get("id")
    ids.append(nid)
    kind = n.get("kind")
    if kind == "desktop":
        desktops += 1
    elif kind == "split":
        kids, weights = n.get("children") or [], n.get("weights") or []
        if len(kids) != len(weights):
            problems.append("%s: %d children vs %d weights" % (nid, len(kids), len(weights)))
        if kids and abs(sum(weights) - 1.0) > 1e-6:
            problems.append("%s: weights sum to %.4f" % (nid, sum(weights)))
        if any(w < 0 for w in weights):
            problems.append("%s: negative weight" % nid)
        for k in kids:
            walk(k)
    elif kind == "panel":
        views = n.get("views") or []
        if not views:
            problems.append("%s: panel with no views" % nid)
if root is None:
    problems.append("no doc.root")
else:
    walk(root)
if desktops != 1:
    problems.append("%d desktop leaves, expected 1" % desktops)
if len(ids) != len(set(ids)):
    problems.append("duplicate node ids")
print("; ".join(problems) if problems else "")
')"
INV_RC=$?
if [ "$INV_RC" -eq 0 ] && [ -z "$INVARIANTS" ]; then
  pass "layout invariants"
  note "  ok   layout invariants"
else
  fail "layout invariants: ${INVARIANTS:-checker failed (rc=$INV_RC)}"
fi

# 4. geometry drift ----------------------------------------------------------
DRIFT=""
DRIFT_COUNT=0
if command -v xdotool >/dev/null && command -v xprop >/dev/null; then
  SCREEN="$(printf '%s\n' "$STATUS" | grep -oE '[0-9]+x[0-9]+' | head -1)"
  SCREEN_W="${SCREEN%x*}"
  SCREEN_H="${SCREEN#*x}"
  : "${SCREEN_W:=99999}" "${SCREEN_H:=99999}"
  "$BIN" rects 2>/dev/null > /tmp/pergola-health-rects.$$
  while read -r id x y w h; do
    [ -z "${id:-}" ] && continue
    win="0x${id#p-}"
    pos=$(xdotool getwindowgeometry "$win" 2>/dev/null | sed -n 's/.*Position: \([-0-9]*\),\([-0-9]*\).*/\1 \2/p')
    size=$(xdotool getwindowgeometry "$win" 2>/dev/null | sed -n 's/.*Geometry: \([0-9]*\)x\([0-9]*\).*/\1 \2/p')
    [ -z "$pos" ] && { DRIFT_COUNT=$((DRIFT_COUNT+1)); continue; }
    ext=$(xprop -id "$win" _NET_FRAME_EXTENTS 2>/dev/null | sed -n 's/.*= \([0-9]*\), \([0-9]*\), \([0-9]*\), \([0-9]*\)/\1 \2 \3 \4/p')
    read -r px py <<<"$pos"; read -r sw sh <<<"$size"; read -r l r t b <<<"${ext:-0 0 0 0}"
    fx=$((px-l)); fy=$((py-t)); fw=$((sw+l+r)); fh=$((sh+t+b))
    # Only assert tiles that are fully on screen: parked tiles are clamped by the WM.
    if [ "$x" -lt 0 ] || [ "$y" -lt 0 ] || $((x+w)) -gt "$SCREEN_W" || $((y+h)) -gt "$SCREEN_H"; then
      continue
    fi
    for d in "$((fx-x))" "$((fy-y))" "$((fw-w))" "$((fh-h))"; do
      if [ "${d#-}" -gt 2 ]; then
        DRIFT_COUNT=$((DRIFT_COUNT+1))
        DRIFT="$DRIFT $id(expected $x,$y ${w}x$h got $fx,$fy ${fw}x$fh)"
        break
      fi
    done
  done < /tmp/pergola-health-rects.$$
  rm -f /tmp/pergola-health-rects.$$
  if [ "$DRIFT_COUNT" -eq 0 ]; then
    pass "no geometry drift"
    note "  ok   no geometry drift"
  else
    fail "geometry drift on $DRIFT_COUNT panel(s):$DRIFT"
  fi
else
  note "  --   drift skipped (xdotool/xprop missing)"
fi

# 5. docked windows that no longer exist ------------------------------------
if command -v wmctrl >/dev/null && [ -n "$STATUS" ]; then
  LIVE="$(wmctrl -l | awk '{print tolower($1)}' | sort -u)"
  ORPHANS=""
  for win in $("$BIN" list 2>/dev/null | awk '$1=="docked"{print tolower($2)}'); do
    printf '%s\n' "$LIVE" | grep -qx "$win" || ORPHANS="$ORPHANS $win"
  done
  if [ -z "$ORPHANS" ]; then
    pass "no orphaned docked windows"
    note "  ok   no orphaned docked windows"
  else
    fail "docked panels whose window is gone:$ORPHANS"
  fi
else
  note "  --   orphan check skipped (wmctrl missing)"
fi

# 6. optional regression suite ----------------------------------------------
if [ "$E2E" -eq 1 ]; then
  if [ ! -x "$ROOT/scripts/e2e.sh" ]; then
    fail "e2e suite not found (set PERGOLA_REPO to the checkout)"
  elif PERGOLA_BIN="$BIN" "$ROOT/scripts/e2e.sh" >/tmp/pergola-e2e-health.log 2>&1; then
    SUMMARY="$(grep -E '^e2e:' /tmp/pergola-e2e-health.log | tail -1)"
    pass "e2e suite ($SUMMARY)"
    note "  ok   e2e suite ($SUMMARY)"
  else
    fail "e2e suite failed (see /tmp/pergola-e2e-health.log)"
  fi
fi

# record ---------------------------------------------------------------------
TS=$(date +%s)
if [ -n "$STATUS" ]; then
  mkdir -p "$(dirname "$LOG")"
  printf '{"ts":%s,"ok":%s,"cpu":%s,"drift":%s,"failures":%s,"e2e":%s}\n' \
    "$TS" "$([ ${#FAILURES[@]} -eq 0 ] && echo true || echo false)" \
    "${CPU:-0}" "$DRIFT_COUNT" \
    "$(printf '%s\n' "${FAILURES[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')" \
    "$E2E" >>"$LOG"
fi

if [ "$JSON" -eq 1 ]; then
  printf '{"ok":%s,"cpu":%s,"drift":%s,"checks":%s,"failures":%s}\n' \
    "$([ ${#FAILURES[@]} -eq 0 ] && echo true || echo false)" "${CPU:-0}" "$DRIFT_COUNT" \
    "$(printf '%s\n' "${CHECKS[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')" \
    "$(printf '%s\n' "${FAILURES[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')"
fi

if [ ${#FAILURES[@]} -gt 0 ]; then
  [ "$QUIET" -eq 1 ] && printf 'pergola health: %d failure(s)\n' "${#FAILURES[@]}" >&2
  exit 1
fi
[ "$QUIET" -eq 1 ] || echo "pergola health: ok"
exit 0
