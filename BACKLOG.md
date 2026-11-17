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

### AUR package: `pergola-git` (0.4.0-dev)

`packaging/aur/PKGBUILD` builds from `main` and installs to system paths: `/usr/bin/pergola`,
`/usr/bin/pergola-health`, `/usr/lib/systemd/user/pergola.service`, the XDG autostart entry, the
LICENSE/NOTICE and the docs. `makepkg -f` then `pacman -U` was run end to end: the file list
matches what `install.sh` writes (the config ships as an example in `/usr/share/doc/pergola-git/`,
since pacman must not write into `$HOME`), `/usr/bin/pergola doctor` was green on the installed
copy, and the package was removed afterwards to leave the user install as the live one.

The backlog asked for `pergola-bin`; that name is a contract in the AUR (`-bin` = prebuilt, no
compilation) and there is no release artifact to download, so shipping it would have meant a
package that lies about itself. The rename and the reason are in `packaging/aur/README.md`; a
`pergola-bin` package is item 2, blocked on a release pipeline.

Two deliberate deviations from the source install: the packaged unit does not hard-code
`DISPLAY=:0` (it inherits the graphical session's environment), and `install.sh` remains the path
for source installs.

### Overlay dim layer (0.4.0-dev)

With a compositor owning `_NET_WM_CM_S0`, `overlay on` draws a full-screen 40% black layer under
the split lines, so 2 px borders read at a glance. The layer carries an empty SHAPE input region,
so pointer events reach the tiles under it (checked at eight screen points, none of which landed
on it). Without a compositor, or without SHAPE, the overlay is borders only and `overlay on`
reports `dim unavailable` rather than claiming a layer it cannot draw. Measured under xfwm4's
compositor: mean screen brightness 0.162 off → 0.098 on → 0.159 off (a 0.60× factor, as 40%
black predicts; the absolute value follows the screen content). `overlay off` destroys the
layer, and 100 toggles leave the root window's child count unchanged with no window carrying
`_NET_WM_WINDOW_OPACITY`. The e2e proves the fallback, then a stand-in selection owner proves the
layer (`scripts/xhelper.py`).

### Config validation and `pergola config` (0.4.0-dev)

`pergola config` prints every key with the value in force and where it came from (`default`, or
`config.json:<line>`), so a misspelled key can no longer hide behind a default. `pergola config
--check` exits non-zero and names the line for an unknown key, an out-of-range
`gap`/`minimum_share`/`desktop_share`, a `terminal` that is not on `PATH`, or a `bindings` chord
that does not parse; it reads the file itself, so it works with no daemon running. The line finder
tracks JSON strings and nesting, so a binding named after a command does not shadow the key, and a
one-line config still names its source. Also fixes the CLI panicking on a closed pipe
(`pergola config | head`), by restoring the default SIGPIPE handler in `main`.

### Settings and chords in the palette (0.4.0-dev)

`pergola settings` opens the same menu programs the action palette uses with one row per config
key and one per command — `gap = 6`, `chord: dock left = Ctrl+Alt+Shift+Left, Super+Shift+Left` —
so the chord map is the `keys --check` view, not a guess. Picking a row asks for a value with the
current one prefilled (`rofi -filter`, `dmenu`, `zenity`/`yad --entry-text`), then applies it
through `pergola settings <key> <value>`, or `pergola bind <command> <chord>` for a chord row.
That primitive validates the whole candidate file with the checks `config --check` runs, writes
atomically and reloads; only problems the change *introduces* block it, so a pre-existing typo
elsewhere does not make the file uneditable (`--check` still reports it, on its line). The menu
path is proven in the e2e with a stand-in dmenu, so no GUI is needed. Still missing: per-key help
in the rows, and a rejected value reported as a notification (UX-NOTES).

### A second window manager in the measured gate (0.4.0-dev)

`scripts/e2e.sh` takes `--wm xfwm4|openbox|i3`, and `pergola-health --e2e` now takes `--wm` too,
forwards it, and records which one ran: the suite's summary line names it (`e2e: 93 passed, 0
failed (wm=openbox)`) and every `health.jsonl` record carries `"wm"`. Two openbox runs of the
whole suite passed with no WM-specific tolerance: the geometry comparison uses each client's own
resize increment (never under 2 px), which already covered the xterm size-hint snap both WMs do,
so no per-WM deviation needed its own number. Run the gate on an idle machine — under CPU
saturation three checks flake (`P list` empty reply, an `arrange` tile left at 0,0, a held chord
that does not step back), and the same three flake at the previous commit, so they are the
suite's fixed sleeps rather than a product regression (`docs/DESIGN.md`).

### Layout undo (0.4.0-dev)

`pergola undo` (bound to `Ctrl+Alt+z`, falling back to `Super+z`) restores the layout from before
the last change: a dock, a float, a drag out to the desktop, an edge drop, an arrange, a gap
change or a gap drag. The ring keeps ten states; each snapshot is the document (tree, floating
rects, tab views) plus the engine's gap and the camera, so undoing a zoomed change puts the camera
back where it was too. A command that changed nothing is not recorded twice, and `undo` itself is
never recorded — otherwise it would oscillate between two states. Windows that closed since the
snapshot are dropped by the reconcile, so undo cannot resurrect a dead seat. `History<T>` lives in
`pergola-core` with its own tests for the bound and the de-duplication; the e2e docks a window by
chord and undoes it.

### Pointer affordances (0.4.0-dev)

The mouse paths are visible now. Hovering a gap band draws a 3 px line down its long axis, so the
6 px strip is findable without hunting; the hint is a separate override-redirect layer with an
empty SHAPE input region, so it never takes the pointer from the band. Dragging a window within
70 px of a work-area edge draws a 4-bar frame around the rectangle the drop would produce: the
preview runs the same three steps as `dock_window` (choose the target, take the seat out of the
tree, insert against the edge), so the frame cannot point somewhere the window will not land. A
docked tile dragged out of its own rectangle and settled near a different edge re-docks there in
one gesture. Hints are skipped when the SHAPE extension is missing; the bands still work. The e2e
asserts the hover hint appears on a gap and leaves no window behind, the drop frame's four bars
under xfwm4 (openbox coalesces a drag into one `ConfigureNotify`, so there is nothing to draw
mid-drag), and the one-gesture re-dock. `HINT_COLOR`, `hint_line`, `frame_bars` and `pointer_edge`
are unit-tested.

### Event stream for status bars (0.4.0-dev)

`pergola subscribe` turns the control socket into a push stream of newline-delimited JSON: a state
snapshot on connect, then `dock`, `undock`, `window-opened`, `window-closed`, `layout` and `focus`
records. Events are diffs of committed state between event-loop passes, so one command's several
mutations collapse into one transition and no event is emitted from a half-applied tree. Writes are
non-blocking and per-client, with a 256-record queue, a remembered partial-write offset (a plain
`write_all` would resend a record after `WouldBlock` and corrupt the stream) and a ten-second
idle drop, so a bar that stops reading cannot stall the window manager — measured at load 5.6: a
subscriber whose stdout went to an undrained fifo left `pergola status` answering in 47 ms. When
the daemon exits, the client prints a note and exits 0 so a supervisor restarts it. Unit tests
cover the transition table (new window, dock, undock, close, layout, focus, no-change); the e2e
reads the whole stream from one session and asserts every required record names the window.
The harvest's **ipc / scripting** class was 227 issues.

### Pager activation requests (0.4.0-dev)

A taskbar's "click to focus" sends `_NET_ACTIVE_WINDOW` to the root window; the daemon only ever
*set* that property when it focused a window, so the click was a silent no-op. The daemon now
selects the root's shared `SubstructureNotify` mask — it cannot take `SubstructureRedirect`, which
the desktop owns — and handles the request: a window in the tree is focused and its seat recorded
as last-focused; a window in a tab group that is not the shown view switches the tab first, so the
desktop never focuses a window nobody can see; a window the daemon does not manage (a stale pager
entry, an override-redirect popup) is ignored rather than focused off the tree. The daemon's own
activation requests echo back through the same selection, so they are tracked for two seconds and
consumed instead of re-handled, which would otherwise bounce focus between the layer and the
desktop forever. Timestamps clients send as `0` are accepted. Unit tests cover the four targeting
cases (shown view, hidden view, floating window, unmanaged id); the e2e asserts an external
`xdotool windowactivate` changes the daemon's active window, that the daemon logged handling the
request itself (an assertion on `status active` alone would pass with the handler deleted, because
the desktop sets the property either way), and that a request for a hidden tab view shows it.
The harvest's **ewmh / icccm** class was 168 issues.

### Named workspaces (tags) on top of views (0.5.0-dev)

A tag is a named set of views over the one tree: `active_tree()` prunes the panels that hold none
of its views and collapses a one-child split, keeping every surviving node id. `pergola window-tag
<name>` moves the focused window onto a workspace (creating it), `tag-view <name>` switches to it,
`tag-view all` shows every view, `tag-view` lists the workspaces, and `tag-remove` deletes a name
without closing its windows. A view belongs to at most one tag (the sets stay disjoint), a tag
remembers the window that had the keyboard, and a window that opens on a shown tag joins it.

A view on a tag that is not shown is *parked* below the work area, keeping its seat, panel, place
in the tree and rectangle, so switching back is exact; the active tag lives in the document and
survives a restart. `status`/`list`/`rects` report the active tag, and the event stream publishes a
`tag` record (with `tag`/`tags` in the state snapshot).

The pruning rule is pure and unit-tested (disjoint sets, collapse of a one-child split, an empty
tag, a tag surviving its last window, a tagged document read by a tags-unaware reader); the e2e
asserts disjoint sets, only the active tag mapped, a parked window mostly off the work area (0%
visible under xfwm4, 7% under openbox — both clamp a window they must not lose),
the escape hatch, a restart and a `tag` record. The document gained `doc.tags`/`doc.active_tag`
behind `serde(default)`, so old documents load unchanged and an older binary reads a tagged
document without losing a view — the `MINOR` bump the item called for. The harvest's
**workspace / tag** class was 236 issues.

Known gaps, recorded rather than hidden: `arrange`/`float-all` rebuild the shared tree and so act
on every tag, the camera and gap are global, tags have no default chords (pressing one on a
desktop with no tag would blank the screen), and a desktop clamps a parked window so a strip stays
reachable at the bottom edge — xfwm4's title bar, openbox's first 66 px of an 862 px client
(`docs/UX-NOTES.md`).

## 1. Release pipeline, so `pergola-bin` can exist

**What:** a CI workflow that builds `x86_64` and `aarch64` binaries on a tag and attaches them to
a GitHub release; then a `pergola-bin` PKGBUILD that downloads that artifact (the file list stays
identical to `pergola-git`).
**Why:** `pergola-git` compiles on the user's machine, which needs a Rust toolchain and ~40 s. A
`-bin` package is the only AUR option for people who do not want that, and the name is only
honest once an artifact exists.
**Done when:** pushing a tag produces a release with both binaries, and a `pergola-bin` PKGBUILD
with a real `sha256sums` installs the same file list as `pergola-git`.

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
  dim layer is the only compositor interaction worth having.
