# Backlog

The development queue. Each item says **what**, **why it matters**, and **done when** — an item
without a testable "done when" is a wish, not a task. Order is deliberately shallow: pick the top
one that is not blocked.

The agenda (`pi-agenda show`, obligations `pergola-dev` every 2d and `pergola-gate` every 14d)
works this list; closing an item here means committing code, tests and a CHANGELOG line.

## 1. Tab groups — make `views > 1` real

**What:** a panel with several views keeps every view mapped at the panel rectangle and raises the
`selected` one; `tab-select <seat>` picks, `tab-next`/`tab-prev` cycle, closing the selected view
promotes the next.
**Why:** the document model already has `views`/`selected`/`tabbed` (a tab group is `views.len() > 1`)
but the daemon only raises and never switches — so the feature exists on paper only.
**Done when:** `pi-agenda`-free e2e assertion: two windows in one panel, both mapped at the same
rect, exactly one raised; `pergola tree` shows `views:[a,b], selected:b`; a unit test covers the
selection repair when the selected view closes.

## 2. Drag the gap between tiles

**What:** grab the 8 px gap with the pointer; drag left/right (or up/down) to change the two
neighbouring weights; `Esc` cancels.
**Why:** `LayoutEngine::resizing_gap` is implemented and tested, but nothing calls it from the
pointer, so resizing means editing JSON or re-docking.
**Done when:** an override-redirect grab window over each gap; `ButtonPress`/`MotionNotify` maps a
pixel delta to a weight delta; the rendered result equals `resizing_gap`'s output for the same
delta (one e2e assertion), and weights never fall below `minimum_share`.

## 3. Auto-dock policy that does not surprise

**What:** `auto_dock` becomes a tri-state: `off` (today), `new` (dock windows opened after the
daemon started), `all` (also adopt windows that appear while running). Document what "the focused
space" means when the active window is the one being docked.
**Why:** the config key exists but the policy has two holes: it only runs at reconcile, and the
target choice is the last focused panel — which is not always what the user expects.
**Done when:** a table test over five cases (first window, second window, focus on desktop, window
that is itself docked, dialog transient) and an e2e run asserting no window is adopted into a
space it was not opened in.

## 4. Packaging: AUR `pergola-bin`

**What:** a PKGBUILD that installs the release binary, the autostart entry, the systemd unit and
`pergola-health`; `install.sh` stays for source installs.
**Why:** "clone and run `install.sh`" is a developer path; a distribution package is the difference
between a project and a tool.
**Done when:** `makepkg -si` on a clean tree installs, `pergola doctor` is green, and the package
lists the same files `install.sh` writes.

## 5. Overlay: dim the unframed region

**What:** when a compositor is present, set `_NET_WM_WINDOW_OPACITY` on a full-screen dim window
under the overlay borders; fall back to borders only when compositing is off.
**Why:** the borders are 2 px and easy to miss; the tree should read at a glance when zoomed out.
**Done when:** `pergola overlay on` shows a dimmed region under xfwm4 with compositing on, nothing
changes with compositing off, and the dim window is destroyed on `overlay off` (no leaked windows
after 100 toggles).

## 6. Second measured window manager in CI

**What:** make `scripts/e2e.sh` take `WM=openbox|xfwm4` and run both; record the size-hint
deviation per WM.
**Why:** the "any EWMH desktop" claim currently rests on two manual runs; a regression under a
different WM would go unnoticed.
**Done when:** `scripts/e2e.sh --wm openbox` passes with the documented xterm size-hint tolerance,
and `pergola-health --e2e` reports which WM it ran.

## 7. Config validation and `pergola config`

**What:** `pergola config` prints the resolved config with its source; `pergola config --check`
rejects unknown keys, out-of-range gaps and a terminal that does not exist.
**Why:** a typo in `config.json` is currently ignored silently (`serde` default), which is a
mystery for the user and a non-bug in the tracker.
**Done when:** a bad key is reported with its line, a good file round-trips, and `--check` exits
non-zero in the failure cases.

## Not planned

- **Wayland-native.** An external process cannot move Wayland clients; anything here would be a
  compositor, which is a different project. XWayland clients only, documented.
- **Screenshot layer for zoom** (TrellisWM's approach). It needs ScreenCaptureKit-class capture
  plus compositing; the geometry transform is the honest X11 equivalent.
- **macOS.** Different window system, different constraints; out of scope.
