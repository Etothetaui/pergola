# Changelog

All notable changes to Pergola. Format follows the versioning rules in
[docs/VERSIONING.md](docs/VERSIONING.md).

## [0.4.0-dev]

### Changed

* **`pergola-health --e2e` takes `--wm` and records it.** The regression suite has
  taken `--wm xfwm4|openbox|i3` since 0.4.0; the health check now forwards a WM
  too (`--wm openbox`), names it in the suite's summary line and adds `"wm"` to
  every `health.jsonl` record, so a green line cannot be mistaken for "measured
  on every desktop". The openbox run of the whole suite passes with no per-WM
  tolerance: the geometry check already compares each client's own resize
  increment (never under 2 px).

### Added

* **The overlay dims the tiles when a compositor can render it.** `overlay on`
  now draws a full-screen 40% black layer under the split lines whenever a
  compositor owns `_NET_WM_CM_S0` — the selection xfwm4, picom, kwin and mutter
  all advertise themselves with — so the 2 px borders read at a glance instead
  of disappearing into the window contents. Measured under xfwm4's compositor:
  mean screen brightness 0.162 with the overlay off, 0.098 with it on, 0.159
  after `overlay off` — a 0.60x factor, which is what 40% black over the
  content predicts (the absolute value depends on what is on screen). The layer
  gets an empty SHAPE input region, so pointer events pass through to the tiles
  (checked at eight screen points: none landed on the dim window); on a server
  without SHAPE, or without a compositor, the overlay stays borders-only and
  `overlay on` says `dim unavailable` instead of
  claiming a layer it cannot draw. `overlay off` destroys the layer; 100 toggles
  leave the root window's child count unchanged and no window carrying
  `_NET_WM_WINDOW_OPACITY`. The e2e proves the fallback with no compositor, then
  starts a stand-in owner and asserts the layer's opacity, click-through and
  destruction (`scripts/xhelper.py`). About 90 assertions, run under both WMs.

* **An AUR package that builds from source.** `packaging/aur/PKGBUILD`
  (`pergola-git`) compiles from `main` and installs to system paths instead of
  your home: `/usr/bin/pergola`, `/usr/bin/pergola-health`,
  `/usr/lib/systemd/user/pergola.service`, the XDG autostart entry, LICENSE,
  NOTICE and the docs. `install.sh` stays the path for source installs. The
  file list mirrors what `install.sh` writes; the one difference is the config,
  which ships as an example in `/usr/share/doc/pergola-git/` because pacman must
  not write into `$HOME`. The packaged unit also drops the hard-coded
  `DISPLAY=:0` and inherits the graphical session's environment instead.
  Verified by building, installing with `pacman -U`, running `pergola doctor`
  green from `/usr/bin`, and removing the package again. The backlog asked for
  `pergola-bin`; that name means *prebuilt binary* in the AUR and no release
  artifact exists, so the honest name is `pergola-git` — the rename and the
  reason are in `packaging/aur/README.md`, and a `-bin` package is now its own
  backlog item, blocked on a release pipeline.

* **`auto_dock` is a tri-state (`off` / `new` / `all`).** `new` docks a window
  opened after the daemon started and leaves the session it found alone; `all`
  also adopts the windows that were already on screen at startup; `off` never
  adopts (the historical default). The old boolean still parses: `false` is
  `off`, `true` is `all`. The dock target is the focused panel only if it is in
  the tree, then the last-focused docked panel, else the whole tree, so the WM
  focusing the new window does not make it its own target. A `DestroyNotify`
  handler drops a dead window's seat immediately: the X server can reuse an XID
  within milliseconds, and on openbox the reused id made a new window look like
  the dead one, so it was never adopted (measured: `auto_dock: new` passed on
  xfwm4 and failed on openbox until the seat was removed on destroy). The unit
  test is the mode table; the e2e asserts a new window floats with `off` and
  docks with `new`. 85/85 on xfwm4 and openbox.

* **Tab groups work.** A panel can hold several windows as tabs: every view is
  mapped at the panel rectangle and only the selected one is raised. `pergola
  tab --window <id> [--target <panel>]` folds a window into a panel as a tab
  (a floating window is folded in from the desktop layer and docked);
  `tab-select <seat|window>` shows one view, `tab-next`/`tab-prev` cycle it
  (bound to `Ctrl+Alt+Page_Down`/`Page_Up`, falling back to `Super+Page_Down`/
  `Page_Up` when the desktop owns the primary). Switching a tab gives the shown
  window the keyboard, so typing goes to the view you can see. Closing the shown
  tab drops that view only — its siblings keep the panel — and promotes the
  neighbour; the group collapses only when the last view goes. The document
  model already had `views`/`selected`/`tabbed`; the daemon just never drove
  them. Unit tests cover selection repair and cycling (wrapping both ways); the
  e2e asserts against X geometry that both tabs are mapped at one rectangle,
  exactly one is raised, the default chord cycles the view, and the survivor
  keeps its tile after its sibling is killed.
* **Keybindings are configurable.** A `bindings` map in `config.json` overrides
  the default chords per command: `{"zoom-out": ["Super+Page_Up"]}` replaces
  that command's candidates, an empty list unbinds it, and commands you do not
  mention keep their defaults. Any command the daemon understands can be bound,
  not only the ones in the default plan (`view 3`, `gap 12`, `frame-desktop`).
  `pergola reload` now re-grabs the hotkeys, so a remap takes effect without a
  restart; `pergola keys --check` prints the chords actually grabbed, `off` for
  a deliberate unbind. Two unit tests cover the merge and the add/unbind paths.
  This is a MINOR bump: a new config key, per [docs/VERSIONING.md](docs/VERSIONING.md).
* **`pergola bind` and `pergola unbind` change a chord without editing JSON.**
  `pergola bind "gap 12" Super+9` writes `config.json` (preserving every other
  setting), reloads the daemon and prints its report, so a conflict appears where
  the command ran instead of only in the journal. `pergola bind <command>` with no
  chord shows the effective candidates and where they come from; `--default`
  removes the override. An unknown command or an unparseable chord is rejected
  before anything is written. A malformed `config.json` is never rewritten — the
  user's typo stays and the error is reported. Four unit tests cover preserve,
  unbind/restore, malformed-file refusal and directory creation.
* **`pergola ratio <0.0-1.0>` rebalances a split without a pointer.** It changes
  the split containing the focused panel and reports the resulting weights
  (`ratio 0.30 · split weights [0.30, 0.70]`); values outside 0–1 are refused.
  `LayoutEngine::set_ratio` clamps to `minimum_share`, so the result matches a
  gap drag of the same delta; the unit test checks the clamp against
  `resizing_gap`, covers a three-child split and rejects NaN. The e2e asserts the
  reported ratio and the new geometry.

### Changed

* **`pergola reload` re-grabs the hotkeys.** It used to re-read the config and
  re-apply geometry without touching the grabs, so changing `hotkeys` or `bindings`
  needed a daemon restart. It now reinstalls the bindings, and `hotkeys: false` at
  reload releases the grabs so the desktop gets its chords back. `reload` now
  returns the grab report (fallbacks and failures) to the caller, so `pergola
  reload` prints it too.

## [0.3.4-dev]

### Added

* **`docs/FIRST-RUN.md` — a five-minute walkthrough for a new user.** Open two
  terminals, dock left and right, then see the rest of the model; a glossary for
  every term the `status` line prints; and a first-run troubleshooting table. The
  README now points to it from Install and from the tree model, and the mouse
  section no longer claims the palette needs no command line.

### Fixed

* `pergola keys --check` printed the *planned* chord for every command, so a
  desktop that owns the primary chord (xfwm4 owns `Ctrl+Alt+arrows`) made it
  claim `bound Ctrl+Alt+Up` while the daemon had actually bound `Super+Up`.
  It now prints the label the daemon really grabbed.
* The daemon announced a chord only when *both* the primary and fallback were
  taken; the README promises a report when a chord falls back to its secondary
  binding. It now prints `pergola: '<command>' bound to <chord> (<primary> is
  owned by the desktop)` at startup. — 2026-11-15
* **`pergola tree` kept a window's old title after a rename.** The `PropertyNotify`
  handler updated the daemon's own seat records but not the document metadata
  that `tree` serialises, so a title change only reached the tree view after the
  next 2 s reconcile or an `arrange`. `sync_meta()` now copies the live seats into
  the document before both `tree` and `save` (`herbstluftwm#35`, `bspwm#50`).
  Verified live: `xdotool set_window --name` then `pergola tree` shows the new
  title immediately.
* **A single-report WM drag is a drag.** Edge docking required three
  `ConfigureNotify` events, which xfwm4 emits and openbox does not (openbox sends
  one ICCCM-synthetic event at the drop position). `MoveWatch` now tracks the last
  applied rect as the drag baseline and treats one event that travelled >= 40 px
  from it as a drag, so a WM that reports only the drop position still docks.
  Two unit tests: `a_single_report_that_travelled_is_a_drag`,
  `a_placement_at_the_edge_is_not_a_drag`.
* **The own-apply guard shrank from 250 ms to 100 ms.** The guard swallows the
  stale `ConfigureNotify` a WM delivers after our move (openbox sends the
  pre-move position after we have already applied the new one; measured at
  ~0.5 ms after the apply). A geometric replacement was tried and failed — the
  stale echo is 803 px from the applied rect, so geometry cannot tell it from a
  real drag. 100 ms still catches the burst while shrinking the window in which
  a real drag can be swallowed 2.5×. Both WMs passed 43/43 three runs each.

### Changed

* **`scripts/e2e.sh` takes `--wm xfwm4|openbox|i3`** (or `PERGOLA_TEST_WM`). The
  "any EWMH desktop" claim is now measurable: the 1.0.0 gate wants every command
  under two window managers. The sandbox refuses to run on a display that already
  has a server, so a kept sandbox cannot masquerade as a product bug. The edge-dock
  test now performs a real pointer drag instead of `wmctrl -e` geometry pokes,
  because each WM coalesces those differently.
* **`scripts/health.sh --e2e` records live-only failures as warnings.** When two
  sessions tile the same live display, the soak's cpu/drift/orphan observations
  fail for reasons that are not the code's. With `--e2e`, those live-only
  observations are written to `health.jsonl` as `warnings` and no longer fail the
  exit code; the 12-hour soak runs without `--e2e` and keeps the strict verdict.

## [0.3.3-dev] — 2026-11-15

### Fixed

* **`pergola keys --check` said every chord was taken while the daemon ran.** The
  probe connected as a fresh client and tried to grab each chord itself, so it
  collided with the daemon's own grabs and reported `TAKEN` for a setup that was
  working. It now asks the running daemon what it actually bound (`keys` over
  IPC) and falls back to the live probe only when no daemon is up. Verified
  against the live daemon: `keys --check` prints `bound` for every chord and says
  it is reporting the daemon's grabs.

## [0.3.2-dev] — 2026-11-14

### Fixed

* **A held chord fires once, not eleven times.** The X server auto-repeats a
  held key (~11 presses a second); `back` held for a second would walk eleven
  history entries. Repeat detection is exact, not a time window: the server
  sends a `KeyRelease` and a `KeyPress` with the *same* timestamp for every
  repeat, so a press matching the preceding release's keycode and timestamp is
  dropped (`awesome#230`). Verified end-to-end by holding `back` for 1.2 s and
  asserting one step (`scripts/e2e.sh`, `PERGOLA_E2E_KEEP=1` keeps the sandbox
  for debugging).
* **A `Super+…` chord no longer dies when Num Lock is on.** The key-event path
  compared the raw modifier state against a mask of `Shift|Control|Mod1`, so a
  grab registered for `Mod4` never matched an event carrying `Mod4|Mod2`. It now
  masks the event state down to the four binding modifiers
  (`pergola_x11::event_mods`), so lock keys cannot break a chord
  (`bspwm#16`, `i3#943`, `spectrwm#103`).
* **Bindings survive a keyboard-layout switch.** Changing to a Cyrillic/Neo/Bone
  layout renumbers keycodes and left the old grabs pointing at the wrong keys.
  `MappingNotify` now triggers `reinstall_bindings()`, which rebuilds the keymap,
  ungrabs everything and re-grabs (`i3#79`, `awesome#157`, `awesome#219`).
* **Grabs are registered for every lock combination.** The Num Lock mask is read
  from the modifier map instead of assumed to be Mod2, and `grab_with_locks()`
  grabs each binding with and without every lock key held.
* **A terminal tile no longer ends up 11 px short of its slot.** Client sizes are
  now fitted to the `WM_NORMAL_HINTS` resize grid by `snap_to_hints()` before the
  frame is placed, so the tile the engine computed and the tile on screen agree
  (`awesome#200`, `awesome#197`).
* **A 0×0 work area no longer collapses the tree.** A RandR mid-change or a
  monitor asleep reports a zero-sized work area; `apply()` now returns early and
  retries on the next pass instead of dividing by zero (`i3#926`).
* **Fullscreen and maximised windows are left alone.** `WindowInfo` now carries
  `_NET_WM_STATE_FULLSCREEN` and the maximised flags, and `apply()`/`note_configure()`
  skip them, so entering fullscreen cannot be mistaken for an edge-dock drag
  (`awesome#245`, `awesome#203`).
* **Transient windows are never tiled.** `WM_TRANSIENT_FOR` is read into
  `WindowInfo.transient`; `reconcile()` will not adopt a dialog into the tree and
  `apply()` skips it, so a completion popup or a file chooser stays where its
  toolkit put it (`awesome#22`, `i3#787`).
* **A minimised window keeps its tile.** Liveness is now decided from
  `_NET_CLIENT_LIST` rather than from the mapped state, so minimising a window no
  longer collapses its tile; `pergola list` reports the seat as `hidden`
  (`i3#718`, `awesome#92`).
* **A window title or class change redraws immediately.** `reconcile()` now
  subscribes to `PROPERTY_CHANGE` and refreshes the title/class on
  `PropertyNotify`, so a browser tab switch or a terminal `cd` does not leave a
  stale label (`herbstluftwm#35`).

### Changed

* `scripts/e2e.sh` compares tile edges against the client's own resize increment
  rather than a fixed ±2 px, matching `scripts/health.sh`, so a snapped terminal
  is not reported as drift. `PERGOLA_E2E_KEEP=1` keeps the sandbox for debugging.

### Added

* `docs/HISTORY-LESSONS.md` — 700 issues from i3, awesomewm, bspwm, xmonad,
  herbstluftwm, qtile and spectrwm, classified into failure classes and mapped to
  the fix or the honest gap. `scripts/research-issues.py` regenerates the harvest
  from the GitHub REST API; `docs/research/` holds the digest and class listing.
* 22 unit tests (was 21) covering the repeat guard, plus one e2e assertion that a
  held chord acts once. Test counts are the measured values (64 unit, 43 e2e).

## [0.3.1-dev] — 2026-11-14

### Fixed

* Docking a **maximised** window silently did nothing: the WM ignores
  `_NET_MOVERESIZE_WINDOW` for a maximised window, so the tile kept the
  full-screen geometry while the tree recorded the tile. It now clears
  `_NET_WM_STATE_MAXIMIZED_{HORZ,VERT}` before the move; `scripts/e2e.sh`
  maximises a window, docks it by chord and asserts the state is gone.
* `pergola-health` geometry drift was wrong in both directions: full-height tiles
  were skipped (`y+h` compared against the work-area height instead of the work
  area's real bottom edge) so a maximised tile could drift unnoticed, and a client
  with resize increments (a terminal) was compared by its frame origin, which the
  WM is free to shift by one increment. It now skips against the work-area
  rectangle and compares the four tile edges with the client's own increment.
  Verified against the live case that prompted the fix.
* Test counts are the measured values (59 unit, 84 e2e).

## [0.3.0-dev] — 2026-11-14

### Added

* **Tiling without a terminal.** `Ctrl+Alt+Shift+←/→/↑/↓` docks the focused window
  left/right/above/below, `Ctrl+Alt+Shift+W` floats it, and `Ctrl+Alt+M` opens an
  action palette (`pergola palette`) that lists the same actions by human name
  through `rofi`, `zenity`, `yad` or `dmenu`. Two focused windows and two chords
  are a side-by-side tiling, with no shell involved.
* **Mouse tiling.** Every split gap is an invisible input-only grab strip with the
  matching resize cursor: drag it to move the boundary (clamped by
  `minimum_share`, exact under the camera transform). Dragging a floating window
  to a work-area edge docks it there (14 px slack), and dragging a docked tile
  away from the edge floats it where it was dropped. The edge behaviour is the
  `edge_dock` config key, default `true`.
* `LayoutEngine::gap_handles()` — the pointer-facing geometry of the tree: one
  band and one weight denominator per draggable boundary. Pure and unit-tested,
  so the daemon never re-derives layout math for the mouse.
* `scripts/e2e.sh` now covers keyboard docking, a real pointer drag on the gap
  and an edge drop followed by a drag-out (82 assertions).

### Changed

* `neighbour up` / `neighbour down` are no longer default chords: the
  `Ctrl+Alt+Shift+arrows` they used are the dock chords now. The command and its
  URL still work (`pergola neighbour up`).

### Fixed

* A docked window that the user drags is no longer left with the tree and the
  screen disagreeing: the pointer path floats it at the drop position, which is
  the inverse of drag-to-tile.

### Documentation

* `docs/UX-NOTES.md` records the human-interface pain points found while building
  this release, each with a severity and a proposed fix, so the next round starts
  from evidence instead of guesses.
* README, `docs/SETUP.md` and `docs/DESIGN.md` describe the keyboard/mouse/palette
  paths, the new config key and the pointer architecture; test counts are the
  measured values (59 unit, 82 e2e).

## [0.2.1-dev] — 2026-11-14

### Fixed

* Rectangles round their **edges**, not their sizes, so a row of tiles can no longer overflow its
  frame by one pixel (873 px split in two gave 434+6+434 = 874).
* `pergola-health` now asserts every tile is inside `_NET_WORKAREA` — the executable guard for the
  "tiled over the panel" class.
* A window that is entirely outside the framed view is parked below the work area instead of to the
  side, where most WMs clamp it back to keep a title-bar sliver reachable.
* The overlay no longer draws a boundary line for a zero-weight child (a collapsed desktop leaf).
* A second daemon no longer unlinks the live socket: it used to steal the socket from a running
  daemon, leaving that daemon alive but unreachable. It now refuses when the socket accepts a
  connection and only reclaims a stale file left by a killed daemon. `scripts/e2e.sh` asserts both
  halves (the refusal and that the first daemon keeps answering).
* `install.sh` writes the installing session's `$DISPLAY` into the systemd user unit instead of
  hard-coding `:0`, so the daemon also starts from a session that is not the first display.

### Documentation

* New [docs/SETUP.md](docs/SETUP.md): requirements, what `install.sh` writes and where, the config
  reference, start-at-login, upgrade, uninstall and troubleshooting. The README now links to it and
  carries the configuration keys inline.
* Test counts in the README and `docs/DESIGN.md` corrected to the measured values (46 unit tests,
  76 e2e assertions).

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
