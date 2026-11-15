# Backlog

The development queue. Each item says **what**, **why it matters**, and **done when** — an item
without a testable "done when" is a wish, not a task. Order is deliberately shallow: pick the top
one that is not blocked.

The agenda (`pi-agenda show`) works this list — `pergola-dev` every 2d, `pergola-gate` every 14d
and the soak/e2e checks; closing an item here means committing code, tests and a CHANGELOG line.

## 1. Tab groups — make `views > 1` real

**What:** a panel with several views keeps every view mapped at the panel rectangle and raises the
`selected` one; `tab-select <seat>` picks, `tab-next`/`tab-prev` cycle, closing the selected view
promotes the next.
**Why:** the document model already has `views`/`selected`/`tabbed` (a tab group is `views.len() > 1`)
but the daemon only raises and never switches — so the feature exists on paper only.
**Done when:** `pi-agenda`-free e2e assertion: two windows in one panel, both mapped at the same
rect, exactly one raised; `pergola tree` shows `views:[a,b], selected:b`; a unit test covers the
selection repair when the selected view closes.

## 2. Auto-dock policy that does not surprise

**What:** `auto_dock` becomes a tri-state: `off` (today), `new` (dock windows opened after the
daemon started), `all` (also adopt windows that appear while running). Document what "the focused
space" means when the active window is the one being docked.
**Why:** the config key exists but the policy has two holes: it only runs at reconcile, and the
target choice is the last focused panel — which is not always what the user expects.
**Done when:** a table test over five cases (first window, second window, focus on desktop, window
that is itself docked, dialog transient) and an e2e run asserting no window is adopted into a
space it was not opened in.

## 3. Packaging: AUR `pergola-bin`

**What:** a PKGBUILD that installs the release binary, the autostart entry, the systemd unit and
`pergola-health`; `install.sh` stays for source installs.
**Why:** "clone and run `install.sh`" is a developer path; a distribution package is the difference
between a project and a tool.
**Done when:** `makepkg -si` on a clean tree installs, `pergola doctor` is green, and the package
lists the same files `install.sh` writes.

## 4. Overlay: dim the unframed region

**What:** when a compositor is present, set `_NET_WM_WINDOW_OPACITY` on a full-screen dim window
under the overlay borders; fall back to borders only when compositing is off.
**Why:** the borders are 2 px and easy to miss; the tree should read at a glance when zoomed out.
**Done when:** `pergola overlay on` shows a dimmed region under xfwm4 with compositing on, nothing
changes with compositing off, and the dim window is destroyed on `overlay off` (no leaked windows
after 100 toggles).

## 5. Second measured window manager in CI

**What:** make `scripts/e2e.sh` take `WM=openbox|xfwm4` and run both; record the size-hint
deviation per WM.
**Why:** the "any EWMH desktop" claim currently rests on two manual runs; a regression under a
different WM would go unnoticed.
**Done when:** `scripts/e2e.sh --wm openbox` passes with the documented xterm size-hint tolerance,
and `pergola-health --e2e` reports which WM it ran.

## 6. Config validation and `pergola config`

**What:** `pergola config` prints the resolved config with its source; `pergola config --check`
rejects unknown keys, out-of-range gaps and a terminal that does not exist.
**Why:** a typo in `config.json` is currently ignored silently (`serde` default), which is a
mystery for the user and a non-bug in the tracker.
**Done when:** a bad key is reported with its line, a good file round-trips, and `--check` exits
non-zero in the failure cases.

## 7. Split ratios without a pointer

**What:** `pergola ratio <0.1-0.9>` rebalances the split that contains the focused panel, plus
palette rows for half / third / two-thirds.
**Why:** the only way to change a split today is to drag its gap with a mouse (0.3.0-dev); a
keyboard user cannot, and `gap: 0` removes the grab strip entirely. `docs/UX-NOTES.md` rows 6.
**Done when:** `pergola ratio 0.3` moves the boundary to 30 % and `pergola rects` shows the
weights; the result is identical to a gap drag of the same delta; a unit test covers the clamp.

## 8. Settings and chords in the palette

**What:** `pergola settings` opens the palette with the config keys (gap, minimum_share,
auto_dock, edge_dock, overlay, terminal) and the chord map; picking a row edits `config.json`
atomically and reloads.
**Why:** the last place a person must edit JSON (UX-NOTES rows 3 and 12). `Command::Settings`
already exists in `pergola-core` and renders nothing.
**Done when:** every key in `config.json` is reachable from the menu, a bad value is rejected
without writing, and `pergola keys --check` equivalents are shown for each chord.

## 9. Layout undo

**What:** a bounded snapshot ring of trees with `Ctrl+Alt+z` / palette "Undo layout change";
redo is out of scope for the first pass.
**Why:** a wrong dock can only be undone by hand (UX-NOTES row 9). The camera's back/forward is
not a layout history and users assume it is.
**Done when:** dock, float, gap drag and arrange each push a snapshot; undo restores the previous
tree, geometry and floating rects; 10 steps are kept; e2e undoes a keyboard dock.

## 10. Pointer affordances

**What:** highlight a gap band on hover even with the overlay off, show the target rectangle during
a window drag, and re-dock a docked tile that settles near another edge.
**Why:** the mouse paths work but are invisible until used (UX-NOTES rows 1, 7, 8). Hover needs
only an EnterNotify on the band; the drop target needs pointer tracking during a drag.
**Done when:** a hovered gap draws a line and leaves no artefact, a dragged window shows where it
will land, and dragging a docked tile to the opposite edge re-docks it in one gesture.

## 11. Event stream for status bars (from `docs/HISTORY-LESSONS.md`)

**What:** `pergola subscribe` streams `layout`, `focus`, `dock`, `undock` and `window-closed`
records as newline-delimited JSON over the control socket, so a panel can draw the tree without
polling `pergola tree`.
**Why:** the harvest's second-largest class is **ipc / scripting** (227 issues) — tilers whose only
interface is "run a command and parse the output" leave every status bar re-implementing state.
Pergola's socket is request/response only today.
**Done when:** a `pergola subscribe` client receives a record on dock, undock and window close,
the stream survives a daemon restart, and an e2e assertion reads two records from one session.

## 12. Honour `_NET_ACTIVE_WINDOW` requests from pagers

**What:** read `_NET_ACTIVE_WINDOW` client messages and focus/raise the named window; the daemon
currently only *sets* that property when it focuses a window.
**Why:** the **ewmh / icccm** class (168 issues) is mostly clients and panels asking the WM to do
something it does not listen for. A taskbar's "click to focus" silently does nothing today.
**Done when:** an external `xdotool windowactivate` on a docked window focuses it and the daemon
reports the same seat as focused; a unit test covers the request path.

## 13. Named workspaces (tags) on top of views

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
  dim planned in item 4 is the only compositor interaction worth having.
