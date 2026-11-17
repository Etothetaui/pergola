# Design

## Shape of the system

```
pergola (binary)                 daemon ── unix socket ── CLI ── hotkeys
   │                                │
   ├── pergola-x11   EWMH: client list, moves, grabs, overlay
   └── pergola-core  pure: tree, geometry, camera, commands, document
```

`pergola-core` has no I/O at all — no X11, no filesystem, no clock — so the
entire model is unit-testable and can be replayed headlessly (`pergola sim`).
`pergola-x11` owns the only unsafe-ish surface: an X connection in safe Rust via
`x11rb`. The binary is the daemon plus a thin client that forwards argv over a
Unix socket.

## Three decisions that matter

### 1. A layer, not a window manager

Pergola never selects `SubstructureRedirect` on the root window, so the running
desktop keeps the display. It reads `_NET_CLIENT_LIST` and asks the WM to move
windows with `_NET_MOVERESIZE_WINDOW` (gravity `NorthWest`, source `pager`) —
the same request `wmctrl` makes. Consequences:

* works inside xfwm4, mutter/X11, kwin_x11, i3, openbox, awesome, …;
* the WM keeps decorations, gravity, size hints and its own constraints;
* the frame rectangle is the unit of layout, so the code converts between frame
  and client coordinates using `_NET_FRAME_EXTENTS` at every move;
* a WM that ignores EWMH moves cannot be supported (`pergola doctor` says so).

### 2. The camera is a transform, not a workspace switch

`framed` is a node id. Rendering the tree means:

```
rects = engine.rects(root, screen)          # world coordinates
if framed != root:                          # zoom
    for each rect: map (rect - framed) * (screen / framed.size)
```

The framed node therefore lands exactly on the screen and everything else moves
with it — a continuous zoom, matching the model. Windows outside the view are
moved off-screen (some WMs clamp them to keep a sliver reachable; documented).

### 3. The desktop is a space, not the screen

Exactly one `desktop` leaf exists in the tree. Floating windows store their
rects in *world* coordinates and are mapped into the desktop leaf's rectangle
whenever it is not the whole screen. Dragging a floating window is converted
back into world coordinates, so the layout learns the new position instead of
snapping the window back. When the desktop leaf moves (a dock, an arrange), the
windows are remapped with it.

## Data flow of one command

```
pergola dock left
  → CLI sends {"argv":["dock","left"]} to the socket
  → daemon parses, finds the focused window's seat, picks a target space
  → engine.inserting(panel, target, Left, root) -> new tree   (pure)
  → tree.validate()                                          (pure)
  → apply(): rects -> camera transform -> _NET_MOVERESIZE_WINDOW per window
  → save(): layout.json                                       (atomic-ish write)
  → CLI prints "docked p-… left of … (depth 2)"
```

`reconcile()` matches the tree against the live client list: seats whose window
is gone are removed (a split with one child collapses into that child), new
windows become floating panels or are auto-docked, and titles are refreshed. It
runs on demand and every two seconds in the event loop.

## Drawing the tree

The overlay is a *divider*, not a border: one 2 px line per split boundary, centred in the gap
between the two children, plus a 3 px amber frame around the work area when a node is framed. Lines
live in the gap, so they never cover a title bar or a window border — the earlier design drew a
border around every tile and read as a debug artefact. It is off by default; `overlay on` (or
`everything`) shows it, and the lines are drawn as override-redirect windows raised above the tiles.

## The human interface: pointer and palette

The engine has no idea whether a command came from a keyboard, a socket, or a
mouse. Two thin layers make that possible, and neither can change the tree on its
own:

### Keyboard

`binding_plan()` in `pergola-x11` maps chords to the same argv the CLI sends
(`"dock left"` is dispatched as `["dock", "left"]`), so a hotkey cannot bypass a
validation the CLI has. A chord the desktop already owns is skipped and the
`Super` alternative is tried; `pergola keys --check` reports the result.

### Pointer

* `LayoutEngine::gap_handles()` is the pointer-facing view of the tree: one hit
  rectangle per gap between adjacent, non-zero siblings, plus the split length a
  pixel delta is divided by. This is pure geometry and unit-tested with the
  engine.
* `X11::grab_bands_show()` turns those rectangles into *input-only*
  override-redirect windows — invisible, no pixels covered, each with the
  matching resize cursor. X's implicit pointer grab means a `ButtonPress` on the
  band keeps the `MotionNotify` stream even though the boundary moves under the
  cursor. The band set is rebuilt on every `apply()`, except during a drag, when
  replacing the window under the pointer would abort it.
* A drag re-derives the tree from the snapshot taken at `ButtonPress`, so the
  result is absolute and cannot accumulate rounding error.
* Dragging a *window* is not visible to an external process; the WM's synthetic
  `ConfigureNotify` (ICCCM, root coordinates) is. The daemon watches every
  managed window, filters its own moves (the rectangle it asked for, within a
  wake window), and waits for three geometry changes followed by 300 ms of quiet.
  A settle within 14 px of a work-area edge docks; anywhere else floats. This
  keeps an application opening at `0,51` from tiling itself.
* A docked tile that is dragged out of its own rectangle and settles near a
  *different* edge re-docks against that edge in one gesture: the same settle
  path calls `float_window` then `dock_window`, so the two-drag detour is gone.
  Requiring the midpoint to leave the old rectangle is what stops a full-width
  row (which touches both left and right) from reading as "already there".
* Two pointer hints answer "what will happen if I let go" without changing the
  tree: `Hint::Hover` draws a 3 px line down the long axis of the gap band under
  the pointer, and `Hint::Drop` draws a 4-bar frame around the rectangle
  `preview_dock()` computes for the edge the pointer is nearest. The preview
  runs the *same* three steps as `dock_window` — choose the target, take the
  seat out of the tree, insert it against the edge — so the frame cannot be
  drawn where the window will not land. Both hints are override-redirect
  windows with an empty SHAPE input region: a hint sits under the pointer by
  definition, so without pass-through the band would get a `LeaveNotify`, take
  the hint down and hand the pointer back in a storm. Without SHAPE the hints
  are skipped entirely and the bands still work. `settle_moves()` clears the
  drop hint (`hide_drop_hint`) but leaves the hover hint alone, because it runs
  on every loop pass and would otherwise erase the hover the band just set.

### Palette

`pergola palette` spawns a detached `palette-run` process (the daemon's event
loop and the CLI must both return immediately) which runs the desktop's menu
program and then calls `pergola <action>`, i.e. the same socket path a hotkey
uses. The action list is static and labelled for people, not for the grammar.

## Failure behaviour

| Situation | Behaviour |
|---|---|
| No `DISPLAY` / no EWMH | `pergola doctor` and the daemon fail with a one-line reason |
| WM ignores `_NET_MOVERESIZE_WINDOW` | window stays put; the CLI still reports success (the request was accepted) |
| Hotkey already owned by the desktop | try the `Super` fallback; report the ones with no free chord at startup |
| Malformed `layout.json` | start from a bare desktop, keep the file untouched |
| Window closes while docked | panel removed, split collapsed, layout re-applied |
| Two daemons | the second refuses: the socket accepts a connection, so it is a live daemon |

## Testing

* `cargo test` — 100 unit tests over geometry, weights, docking, removal, gap
  clamping, ratios, neighbours, camera history, saved views, command URLs,
  document round-trips, config parsing and repair of malformed weights,
  pointer-edge picking and hint geometry.
* `scripts/e2e.sh` — starts Xvfb + a real WM (xfwm4 by default, openbox with
  `--wm openbox`) + three xterms, runs the real daemon, and asserts the geometry
  the X server reports against `pergola rects` after dock, zoom, arrange, gap
  changes, ratio, float, a daemon restart, keyboard docking, a gap drag, an edge
  drop, tab groups, config/binding errors, the overlay's dim layer, hover
  hints, the drop frame and one-gesture re-dock — 124 assertions, a check or
  two fewer when the WM parks a tile off-screen. Openbox coalesces a drag into
  one `ConfigureNotify`, so the mid-drag drop frame is asserted on xfwm4 only;
  the frame's existence is a WM capability, not a tolerance.
  It never touches the live session.

  The suite uses fixed sleeps, so it assumes the machine is not CPU-saturated.
  Measured 2026-11-16 with a local model holding both cores (load 6): three
  checks flaked — an empty `P list` reply, an `arrange` tile left at 0,0, and a
  held chord that did not step back — and the same three flaked at the previous
  commit, so they are the suite's timing, not a product regression. Run the gate
  on an idle machine; a green run is 0 failed on both WMs.
