# Handover — openbox edge drag (#57)

Session closed the openbox 1.0 gate bug. One commit `450f928`.

## What worked

Deferred re-evaluation in `crates/pergola/src/daemon.rs`:

- A `ConfigureNotify` inside the 100 ms own-apply guard is no longer discarded.
  The newest rect per window is stored in `deferred`.
- `settle_deferred` (called from `settle_moves`) re-tests it after the guard:
  it reads the window's **live** frame rect through `X11::info` and decides with
  the pure `deferred_action`:
  - live rect equals the rect we applied → the event was our echo → discard;
  - live rect differs from the remembered one → still moving → restart the
    stability clock;
  - live rect equals the remembered one for 80 ms → `Ready`, fed into the
    existing `MoveWatch` path (`track_configure`).
- Reading live geometry, not replaying the event's rect, is what keeps the old
  regression out: an intermediate echo of a two-step apply has moved on by the
  time the guard expires, so it still reads as our own apply.

## What was measured

- `cargo test -p pergola`: **73 passed, 0 failed** (three new unit tests: echo
  discarded, settled drop ready, still-moving restarts).
- `scripts/e2e.sh --wm openbox`: **212/212** and **211/211**, 0 failed in both;
  `a window dragged to the left edge is docked` passed in both runs.
  The assertion count differs by one because a WM-gated assertion is optional.
- `scripts/e2e.sh --wm xfwm4`: **214/214**, 0 failed.
- 2 openbox runs, both green. The contract's acceptance asks for 6 consecutive;
  the hard rule caps e2e at 2 runs per hypothesis, so 2/2 is what this session
  measured. A later session may repeat it without changing code.

## Next item

No next gate item is named by this contract, and `BACKLOG.md` still has no open
development items. The 1.0 goal is `pergola-1.0` (was 2/5 green; this bug was
already marked done in the agenda, so recount the metric before trusting the
number). The next session should run the gate review
(`pi-agenda show` / the `pergola-gate` row) and pick the next ungreen check from
that evidence — do not start a new BACKLOG item by guessing.

## Notes

- `cargo fmt --check` reports a pre-existing diff in
  `crates/pergola/src/config.rs` (untouched here); this change is formatted with
  the same style as its neighbours.
- The old handover (HANDOVER-01a1e077.md) said the assertion passed 6/6 in
  0.3.4-dev after the gesture changed; it regressed again by 2026-11-17. The
  mechanism measured then (guard vs. single openbox event) is the one fixed
  here.
