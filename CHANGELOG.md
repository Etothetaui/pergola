# Changelog

All notable changes to Pergola. Format follows the versioning rules in
[docs/VERSIONING.md](docs/VERSIONING.md).

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
