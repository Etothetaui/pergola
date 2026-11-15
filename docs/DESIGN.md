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

## Failure behaviour

| Situation | Behaviour |
|---|---|
| No `DISPLAY` / no EWMH | `pergola doctor` and the daemon fail with a one-line reason |
| WM ignores `_NET_MOVERESIZE_WINDOW` | window stays put; the CLI still reports success (the request was accepted) |
| Hotkey already owned by the desktop | try the `Super` fallback; report the ones with no free chord at startup |
| Malformed `layout.json` | start from a bare desktop, keep the file untouched |
| Window closes while docked | panel removed, split collapsed, layout re-applied |
| Two daemons | the second fails to bind the socket |

## Testing

* `cargo test` — 44 unit tests over geometry, weights, docking, removal, gap
  clamping, neighbours, camera history, saved views, command URLs, document
  round-trips and repair of malformed weights.
* `scripts/e2e.sh` — starts Xvfb + xfwm4 + three xterms, runs the real daemon,
  and asserts the geometry the X server reports against `pergola rects` after
  dock, zoom, arrange, gap changes, float and a daemon restart (75 assertions).
  It never touches the live session.
