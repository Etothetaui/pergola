# Backlog

The development queue. Each item says **what**, **why it matters**, and **done when** — an item
without a testable "done when" is a wish, not a task. Order is deliberately shallow: pick the top
one that is not blocked.

The agenda (`pi-agenda show`) works this list — `pergola-dev` every 2d, `pergola-gate` every 14d
and the soak/e2e checks; closing an item here means committing code, tests and a CHANGELOG line.

## Done

### Tab groups — `views > 1` is real (0.4.0-dev)

A panel with several views keeps every view mapped at the panel rectangle and raises the
`selected` one; `tab --window <id>` folds a window in, `tab-select <seat|window>` picks,
`tab-next`/`tab-prev` cycle (default chords `Ctrl+Alt+Page_Down`/`Page_Up`, `Super+…` fallback),
and closing the selected view promotes the next. Switching a tab activates the shown window, so
the keyboard follows the view. `Panel::select`/`cycle`/`remove_view` and
`LayoutEngine::removing_view` are unit-tested; the e2e asserts both views at one rect, exactly one
raised, `tree` reporting `selected`, a real chord press, and that closing one tab leaves its
sibling tiled. 75/75 on xfwm4 and openbox.

### Split ratios without a pointer (0.4.0-dev)

`pergola ratio <0.0-1.0>` rebalances the split that contains the focused panel and reports the
resulting weights (`ratio 0.30 · split weights [0.30, 0.70]`). Values outside 0–1 are refused, and
`LayoutEngine::set_ratio` clamps to `minimum_share`, matching a gap drag of the same delta. The
unit test checks the clamp against `resizing_gap` and rejects NaN; the e2e asserts the reported
ratio and the new geometry. Palette rows for half/third and a nudge chord are still open (see
the Ratios row in `docs/UX-NOTES.md`).

### Auto-dock policy that does not surprise (0.4.0-dev)

`auto_dock` is a tri-state — `off`, `new`, `all` — and the old boolean still parses (`false` is
`off`, `true` is `all`). `new` docks a window that appears after the daemon's first reconcile and
leaves the session it found alone; `all` also adopts the windows that were already on screen;
`off` never adopts. The target is the focused panel *only if it is in the tree*; otherwise the
last-focused docked panel, otherwise the whole tree, so the WM having focused the new window
does not make it its own docking target. A `DestroyNotify` handler removes a dead window's seat
immediately instead of waiting for the 2 s reconcile: the X server reuses the XID, and a reused
id was mistaken for the dead window and never adopted (measured on openbox). The unit test is a
table over the modes; the e2e asserts a new window floats with `off` and docks with `new`. 85/85
on xfwm4 and openbox.

## 1. Packaging: AUR `pergola-bin`

**What:** a PKGBUILD that installs the release binary, the autostart entry, the systemd unit and
`pergola-health`; `install.sh` stays for source installs.
**Why:** "clone and run `install.sh`" is a developer path; a distribution package is the difference
between a project and a tool.
**Done when:** `makepkg -si` on a clean tree installs, `pergola doctor` is green, and the package
lists the same files `install.sh` writes.

## 2. Overlay: dim the unframed region

**What:** when a compositor is present, set `_NET_WM_WINDOW_OPACITY` on a full-screen dim window
under the overlay borders; fall back to borders only when compositing is off.
**Why:** the borders are 2 px and easy to miss; the tree should read at a glance when zoomed out.
**Done when:** `pergola overlay on` shows a dimmed region under xfwm4 with compositing on, nothing
changes with compositing off, and the dim window is destroyed on `overlay off` (no leaked windows
after 100 toggles).

## 3. Second measured window manager in CI

**What:** make `scripts/e2e.sh` take `WM=openbox|xfwm4` and run both; record the size-hint
deviation per WM.
**Why:** the "any EWMH desktop" claim currently rests on two manual runs; a regression under a
different WM would go unnoticed.
**Done when:** `scripts/e2e.sh --wm openbox` passes with the documented xterm size-hint tolerance,
and `pergola-health --e2e` reports which WM it ran.

## 4. Config validation and `pergola config`

**What:** `pergola config` prints the resolved config with its source; `pergola config --check`
rejects unknown keys, out-of-range gaps and a terminal that does not exist.
**Why:** a typo in `config.json` is currently ignored silently (`serde` default), which is a
mystery for the user and a non-bug in the tracker.
**Done when:** a bad key is reported with its line, a good file round-trips, and `--check` exits
non-zero in the failure cases.

## 5. Settings and chords in the palette

**What:** `pergola settings` opens the palette with the config keys (gap, minimum_share,
auto_dock, edge_dock, overlay, terminal) and the chord map — the `bindings` map that has
existed in `config.json` since 0.4.0-dev; picking a row edits `config.json` atomically and
reloads.
**Why:** the last place a person must edit JSON (UX-NOTES rows 3 and 12). `Command::Settings`
already exists in `pergola-core` and renders nothing.
**Done when:** every key in `config.json` is reachable from the menu, a bad value is rejected
without writing, and `pergola keys --check` equivalents are shown for each chord.

## 6. Layout undo

**What:** a bounded snapshot ring of trees with `Ctrl+Alt+z` / palette "Undo layout change";
redo is out of scope for the first pass.
**Why:** a wrong dock can only be undone by hand (UX-NOTES row 9). The camera's back/forward is
not a layout history and users assume it is.
**Done when:** dock, float, gap drag and arrange each push a snapshot; undo restores the previous
tree, geometry and floating rects; 10 steps are kept; e2e undoes a keyboard dock.

## 7. Pointer affordances

**What:** highlight a gap band on hover even with the overlay off, show the target rectangle during
a window drag, and re-dock a docked tile that settles near another edge.
**Why:** the mouse paths work but are invisible until used (UX-NOTES rows 1, 7, 8). Hover needs
only an EnterNotify on the band; the drop target needs pointer tracking during a drag.
**Done when:** a hovered gap draws a line and leaves no artefact, a dragged window shows where it
will land, and dragging a docked tile to the opposite edge re-docks it in one gesture.

## 8. Event stream for status bars (from `docs/HISTORY-LESSONS.md`)

**What:** `pergola subscribe` streams `layout`, `focus`, `dock`, `undock` and `window-closed`
records as newline-delimited JSON over the control socket, so a panel can draw the tree without
polling `pergola tree`.
**Why:** the harvest's second-largest class is **ipc / scripting** (227 issues) — tilers whose only
interface is "run a command and parse the output" leave every status bar re-implementing state.
Pergola's socket is request/response only today.
**Done when:** a `pergola subscribe` client receives a record on dock, undock and window close,
the stream survives a daemon restart, and an e2e assertion reads two records from one session.

## 9. Honour `_NET_ACTIVE_WINDOW` requests from pagers

**What:** read `_NET_ACTIVE_WINDOW` client messages and focus/raise the named window; the daemon
currently only *sets* that property when it focuses a window.
**Why:** the **ewmh / icccm** class (168 issues) is mostly clients and panels asking the WM to do
something it does not listen for. A taskbar's "click to focus" silently does nothing today.
**Done when:** an external `xdotool windowactivate` on a docked window focuses it and the daemon
reports the same seat as focused; a unit test covers the request path.

## 10. Named workspaces (tags) on top of views

**What:** named tags that own a set of views, with `tag-view <name>`, `window-tag <name>` and
per-tag focus memory; the document format gains a `tags` list beside `views`.
**Why:** the largest class in the harvest is **workspace / tag** (236 issues), and Pergola's views
are anonymous: there is no way to say "move this window to view 3". This is a format change, so it
is a deliberate `MINOR` bump with a loader that keeps reading older documents.
**Done when:** two tags hold disjoint view sets, switching tags maps only the active tag's views,
and a document written by the new version loads in the old one without losing its views.

## Not planned

- **Wayland-native.** An external process cannot move Wayland clients; anything here would be a
  compositor, which is a different project. XWayland clients only, documented.
- **Screenshot layer for zoom** (TrellisWM's approach). It needs ScreenCaptureKit-class capture
  plus compositing; the geometry transform is the honest X11 equivalent.
- **macOS.** Different window system, different constraints; out of scope.
- **Tray and panel ownership** (132 issues in the harvest). Pergola reads the work area a
  desktop's panel leaves; it does not draw a bar or own a tray. A tiler that owns the bar is a
  desktop environment, not a tiler.
- **Compositing** (57 issues). Opacity and effects belong to the running compositor; the overlay
  dim planned in item 2 is the only compositor interaction worth having.
