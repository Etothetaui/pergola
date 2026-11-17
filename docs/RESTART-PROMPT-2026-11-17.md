GOAL: close the next ungreen pergola 1.0 gate item. Start with bug #57: under openbox the
e2e assertion "a window dragged to the left edge is docked" fails (xfwm4 is 43/43, openbox 42/43).

STATE (verified 2026-11-17 15:45Z, do not re-derive it):
- ~/Projects/pergola, branch main, working tree clean, HEAD 23f934c, version 0.5.0-dev.
- BACKLOG.md has no open development items left: 19 items, all under "## Done".
- pi-agenda goal pergola-1.0 is at 2/5 gate items green; the release pipeline is blocked on a tag,
  docs/VERSIONING.md forbids a tag before 1.0.0.
- The previous 49-hour session ended in a verification loop: 20 identical `git status; git log`
  calls in 70 seconds with nothing to change, because its backlog was empty.

INPUT: HANDOVER-01a1e077.md, section "What did NOT work (measured, do not retry blind)". The
openbox drop event is discarded by note_configure's `last_apply.elapsed() < 250 ms` guard; removing
the guard docks every window (e2e went 1 -> 7 failures), so the guard stays. The untested fix is a
*deferred* re-evaluation: remember the rect of an event that arrives inside the guard window and
re-test it once the window has expired.

DO:
1. Implement deferred re-evaluation in crates/pergola/src/daemon.rs (MoveWatch already exists).
2. Unit-test it; run `scripts/e2e.sh --wm openbox` and `scripts/e2e.sh --wm xfwm4`.
3. Commit code + tests + a CHANGELOG line; move nothing else in BACKLOG except marking the item.
4. Then stop — do not start the next item.

HARD RULES:
- At most 2 e2e runs per hypothesis (once to reproduce, once to verify). Never launch e2e.sh in a loop.
- Never `git checkout --` your own edit to re-apply the same patch. If an edit fails to match,
  re-read the file and change the anchor.
- If `git status` is clean, that fact is settled: do not check it again in that turn.
- After 3 failed hypotheses, write what you measured into BACKLOG.md under the openbox item and stop.
- Delegate read-only checks (verifier/tester) instead of re-running the suite yourself.

STOP: committed-and-green, or 90 minutes, whichever comes first. Then write HANDOFF-LIVE.md with
what worked, what was measured, and the next item. Acceptance: openbox edge-drag assertion passes 6
consecutive runs, xfwm4 stays 43/43, one commit, BACKLOG and CHANGELOG updated.
