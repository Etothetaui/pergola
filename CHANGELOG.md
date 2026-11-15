# Changelog

All notable changes to Pergola. Format follows the versioning rules in
[docs/VERSIONING.md](docs/VERSIONING.md).

## [0.2.0-dev] — 2026-11-14

### Fixed

* Tiles use the work area (`_NET_WORKAREA` for the current desktop) instead of the whole display,
  so a top panel no longer pushes the bottom of every tile off-screen. The camera transform, and
  the desktop-to-world mapping for floating windows, now respect a work area that does not start
  at `0,0`; `pergola rects` reports frame coordinates that `pergola-health` can compare directly.
* A zero-weight child (a collapsed desktop leaf) no longer reserves a gap, so docking beside the
  desktop leaves no black strip at the screen edge.
* `pergola-health` tolerates a client's own resize increments (terminals snap to a character grid)
  instead of reporting them as drift.

### Changed

* **The overlay is a divider, not a border.** It now draws one 2 px line per split boundary,
  centred in the gap, plus a 3 px amber frame around the viewport when a node is framed. The old
  design drew a border around every tile, which sat on top of title bars and read as a debug
  artefact. Default is **off**; `overlay on` or `everything` shows it.
* Default gap 8 → **6 px**.
* Docking beside the desktop gives the docked window the whole space by default
  (`desktop_share = 0`); set `desktop_share` in the config to keep a visible desktop strip.
* The persisted `overlay` flag no longer overrides the config on startup.

## [0.1.0-dev] — 2026-11-14

### Added

* `scripts/health.sh` (installed as `pergola-health`): daemon liveness, CPU, layout invariants,
  geometry drift against the X server, orphaned docked windows, and an optional `--e2e` pass.
  Exits non-zero and appends one JSON line per run, so a scheduler can act on it.
* `BACKLOG.md`: the development queue, every item with a testable "done when".
* `scripts/e2e.sh` asserts that a floating window's geometry survives a zoom.

## [0.0.0-dev] — 2026-11-14

First working build, installed and tested on Arch Linux / XFCE (xfwm4) and in an
Xvfb + xfwm4 sandbox.

### Added

* `pergola-core`: recursive split tree (`Desktop`/`Panel`/`Split`), geometry with
  gaps, docking on four edges (same-axis sibling insert, cross-axis nesting,
  root wrapping), removal with split collapse, gap resizing with a minimum
  share, neighbour search, camera with back/forward history and nine saved
  views, command vocabulary and `pergola://` URLs, `version: 2` document with
  tolerant loading and weight repair.
* `pergola-x11`: EWMH client-list enumeration, frame-aware moves through
  `_NET_MOVERESIZE_WINDOW`, `_NET_ACTIVE_WINDOW`, override-redirect tree overlay,
  hotkey grabs with a `Super` fallback when the desktop owns a chord,
  `pergola doctor` diagnostics.
* `pergola` daemon: tree/window reconciliation (on demand and every 2 s),
  camera transform, desktop-leaf mapping for floating windows, layout
  persistence, Unix-socket IPC, CLI with 25 subcommands.
* `install.sh`, XDG autostart entry, systemd user unit.
* 37 unit tests; `scripts/e2e.sh` with 78 assertions on a real X server.
