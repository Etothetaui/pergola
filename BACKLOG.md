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
`pergola-bin` package is item 6, blocked on a release pipeline.

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

## 1. Layout undo

**What:** a bounded snapshot ring of trees with `Ctrl+Alt+z` / palette "Undo layout change";
redo is out of scope for the first pass.
**Why:** a wrong dock can only be undone by hand (UX-NOTES row 9). The camera's back/forward is
not a layout history and users assume it is.
**Done when:** dock, float, gap drag and arrange each push a snapshot; undo restores the previous
tree, geometry and floating rects; 10 steps are kept; e2e undoes a keyboard dock.

## 2. Pointer affordances

**What:** highlight a gap band on hover even with the overlay off, show the target rectangle during
a window drag, and re-dock a docked tile that settles near another edge.
**Why:** the mouse paths work but are invisible until used (UX-NOTES rows 1, 7, 8). Hover needs
only an EnterNotify on the band; the drop target needs pointer tracking during a drag.
**Done when:** a hovered gap draws a line and leaves no artefact, a dragged window shows where it
will land, and dragging a docked tile to the opposite edge re-docks it in one gesture.

## 3. Event stream for status bars (from `docs/HISTORY-LESSONS.md`)

**What:** `pergola subscribe` streams `layout`, `focus`, `dock`, `undock` and `window-closed`
records as newline-delimited JSON over the control socket, so a panel can draw the tree without
polling `pergola tree`.
**Why:** the harvest's second-largest class is **ipc / scripting** (227 issues) — tilers whose only
interface is "run a command and parse the output" leave every status bar re-implementing state.
Pergola's socket is request/response only today.
**Done when:** a `pergola subscribe` client receives a record on dock, undock and window close,
the stream survives a daemon restart, and an e2e assertion reads two records from one session.

## 4. Honour `_NET_ACTIVE_WINDOW` requests from pagers

**What:** read `_NET_ACTIVE_WINDOW` client messages and focus/raise the named window; the daemon
currently only *sets* that property when it focuses a window.
**Why:** the **ewmh / icccm** class (168 issues) is mostly clients and panels asking the WM to do
something it does not listen for. A taskbar's "click to focus" silently does nothing today.
**Done when:** an external `xdotool windowactivate` on a docked window focuses it and the daemon
reports the same seat as focused; a unit test covers the request path.

## 5. Named workspaces (tags) on top of views

**What:** named tags that own a set of views, with `tag-view <name>`, `window-tag <name>` and
per-tag focus memory; the document format gains a `tags` list beside `views`.
**Why:** the largest class in the harvest is **workspace / tag** (236 issues), and Pergola's views
are anonymous: there is no way to say "move this window to view 3". This is a format change, so it
is a deliberate `MINOR` bump with a loader that keeps reading older documents.
**Done when:** two tags hold disjoint view sets, switching tags maps only the active tag's views,
and a document written by the new version loads in the old one without losing its views.

## 6. Release pipeline, so `pergola-bin` can exist

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
  dim done in item 1 is the only compositor interaction worth having.
