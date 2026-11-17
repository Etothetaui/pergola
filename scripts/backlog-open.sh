#!/usr/bin/env bash
# Is there development work left in BACKLOG.md? One answer, for a scheduler.
#
#   scripts/backlog-open.sh           exit 0 + "N open" when an item is open, exit 1 + reason when not
#   scripts/backlog-open.sh --list    one line per open item ("<section> :: <title>")
#
# Why: the `pergola-dev` agenda row used to open a session every 2 days whether or not the
# backlog had anything in it. On 2026-11-17 that produced a 49-hour session that closed all 19
# items and then re-verified a clean tree 20 times because nothing told it the queue was empty
# (bug #198). This is the check that answers "is there work" so the row can skip instead of spend.
#
# An item is open when it is a "### " heading that is NOT under "## Done", "## Not planned", or a
# "## Blocked" section. Blocked items carry their remaining condition in the section title (the
# release pipeline is blocked on a tag docs/VERSIONING.md forbids before 1.0.0), so they are not
# work a session may start.
set -uo pipefail

repo=$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/.." && pwd)
backlog="${PERGOLA_BACKLOG:-$repo/BACKLOG.md}"
[ -r "$backlog" ] || { echo "no readable BACKLOG.md at $backlog"; exit 2; }

list=$(awk '
  /^## /   { section = $0; open = (section !~ /^## (Done|Not planned|Blocked)/); next }
  /^### /  { if (open) print section " :: " substr($0, 5) }
' "$backlog")

n=$(printf '%s\n' "$list" | grep -c '::' || true)
[ "$n" -eq 0 ] && n=0

if [ "${1:-}" = "--list" ]; then
  [ "$n" -gt 0 ] && printf '%s\n' "$list" | grep '::'
fi

if [ "$n" -gt 0 ]; then
  echo "$n open"
  exit 0
fi
echo "BACKLOG.md has no open item (all under Done / Blocked / Not planned); nothing to start"
exit 1
