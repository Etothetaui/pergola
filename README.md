# Pergola

A **fractal window manager layer for X11 desktops** — windows nest inside windows
as deep as you like, and a camera lets you zoom into the part you need.

Pergola is a Rust reimplementation of the model popularised by
[TrellisWM](https://trelliswm.app) for macOS, rebuilt as a Linux window-manager
*layer* rather than a replacement: it never takes over the display, so it runs
inside xfwm4, mutter (X11), kwin_x11, i3, openbox, awesome, or any other
EWMH-compliant window manager.

![Pergola: one tile on the left, a nested column of three on the right, and the desktop space](docs/screenshot.png)

```text
 root ── split(x) ─┬─ p-editor ────────────────────────┐
                   ├─ split(y) ─┬─ p-browser ──────────┤
                   │            └─ p-terminal ─────────┤
                   └─ desktop (floating windows) ──────┘
```

## What this is (and is not)

* **Is**: a real, installed, tested window manager *layer*. It docks, nests,
  resizes, zooms, saves views and restores layouts on a running X11 desktop.
* **Is not**: a fork, port or derivative of TrellisWM's code. TrellisWM is
  closed-source; Pergola is a clean-room implementation of the *observable
  model* (recursive split tree + camera), derived from public documentation and
  a public clean-room study. See [docs/RESEARCH.md](docs/RESEARCH.md) and
  [NOTICE](NOTICE) for provenance.
* **Is not**: a Wayland compositor. Wayland clients cannot be moved by an
  external process; Pergola needs X11 (or XWayland, where it can manage X11
  clients only).

## Install

```bash
git clone https://github.com/SyntheticPrometheus/pergola
cd pergola
./install.sh            # build, install to ~/.local/bin, autostart, start now
```

Requirements: Rust (1.75+), an X11 session, `xdotool`/`wmctrl` only for the
optional test script. No root, no daemon privileges, no Accessibility-style
permission prompt. `install.sh` installs `pergola` and `pergola-health` into
`~/.local/bin`, writes `~/.config/pergola/config.json` (only if absent), and
adds a systemd user unit plus an XDG autostart entry so the daemon starts with
your session. `./install.sh --no-start` skips the start.

```bash
pergola doctor          # display, EWMH client list, running WM
pergola status          # the daemon's view: screen, panels, depth, gap, overlay
pergola keys --check    # which chords are free on this desktop
pergola-health          # soak check; exits non-zero on any failure
```

Full install, upgrade, uninstall and troubleshooting reference:
**[docs/SETUP.md](docs/SETUP.md)**.

## Quick start

```bash
pergola dock left        # dock the focused window to the left of its space
pergola dock right       # dock the next window beside it
pergola dock down        # nesting: a split inside a split
pergola frame-window     # frame the focused window (again to go back)
pergola zoom-out         # step one level out
pergola arrange          # tile every window into a balanced fractal tree
pergola float-all        # give every window back to the desktop layer
pergola tree             # the layout document, as JSON
pergola rects            # every panel's current frame rectangle
```

## Commands

| Command | Meaning |
|---|---|
| `everything` | show the whole tree (zoom all the way out) |
| `zoom-in` / `zoom-out` | step one level in / out |
| `back` / `forward` | browser-style camera history |
| `frame-window` / `frame-desktop` | frame the focused window / the desktop layer |
| `toggle-desktop` | hide or show floating windows |
| `neighbour <left\|right\|up\|down>` | move to the neighbouring space |
| `view <1-9>` / `save-view <1-9>` | saved views |
| `dock <edge> [--window ID] [--target ID]` | dock a window against another space |
| `float [--window ID]` | return a window to the desktop layer |
| `arrange` / `float-all` | tile everything / float everything |
| `split-terminal` | split the focused window with a new terminal |
| `gap <px>` | gap between tiles |
| `overlay [on\|off\|toggle]` | draw split lines in the gaps (off by default) |
| `status` `tree` `list` `rects` `keys` `doctor` `sim` | inspection |
| `reload` `save` `stop` | daemon control |

Every command also has a URL form: `pergola://zoom-in`,
`pergola://neighbour?dir=left`, `pergola://view?n=3`.

## Hotkeys

Primary bindings mirror TrellisWM's `⌃⌥` shortcuts; when the desktop already
owns a chord (xfwm4 owns `Ctrl+Alt+arrows` and `Ctrl+Alt+D`), Pergola falls back
to `Super` for that command and reports it at startup.

| Keys | Command |
|---|---|
| `Ctrl+Alt+Space` | everything |
| `Ctrl+Alt+↑` / `↓` | zoom-out / zoom-in |
| `Ctrl+Alt+←` / `→` | neighbour left / right |
| `Ctrl+Alt+Shift+↑` / `↓` | neighbour up / down |
| `Ctrl+Alt+[` / `]` | back / forward |
| `Ctrl+Alt+Return` | frame-window |
| `Ctrl+Alt+D` | toggle-desktop |
| `Ctrl+Alt+N` | split-terminal |
| `Ctrl+Alt+A` | arrange |
| `Ctrl+Alt+F` | float-all |
| `Ctrl+Alt+O` | overlay |

`Super+…` is the automatic fallback for any chord the desktop already uses.
`pergola keys --check` prints what was actually bound and which chords the
desktop already owns.

## Configuration

`~/.config/pergola/config.json` (all keys optional):

| Key | Default | Effect |
|---|---|---|
| `gap` | `6.0` | pixels between tiles and width of the overlay divider |
| `minimum_share` | `0.05` | smallest fraction a tile can be squeezed to |
| `auto_dock` | `false` | dock new windows automatically instead of floating them |
| `overlay` | `false` | draw the split lines and viewport frame |
| `desktop_share` | `0.0` | desktop strip `arrange` keeps; `0` gives it none |
| `terminal` | `$TERMINAL`, else `xfce4-terminal` | program used by `split-terminal` |
| `hotkeys` | `true` | grab the default chords on the root window |

`pergola reload` re-reads the file; `overlay` and `hotkeys` apply on daemon
restart. The `gap` and `overlay` commands change the running session only. See
[docs/SETUP.md](docs/SETUP.md#configuration) for the details.

## How it works

The window tree is the whole model:

```text
Node    := Desktop | Panel | Split
Desktop := { kind:"desktop", id:"desktop" }            # the floating layer
Panel   := { kind:"panel", id, views:[SeatID], selected, tabbed? }
Split   := { kind:"split", id, axis:"x"|"y", children:[Node], weights:[f64] }
```

* A `Split` may contain `Split`s. That recursion is the fractal part.
* Exactly one `Desktop` leaf exists; it is a *space*, not the screen, and
  floating windows live inside whatever rectangle it currently occupies.
* The camera is just a framed node id plus back/forward stacks and nine saved
  views. Framing a node maps it onto the screen and everything else moves with
  it — a real, continuous zoom, not a workspace switch.
* Geometry is applied through `_NET_MOVERESIZE_WINDOW`, the same EWMH request
  `wmctrl` uses, so the running WM keeps its decorations, gravity and
  constraints intact.
* The tree, camera and window fingerprints persist to
  `${XDG_STATE_HOME:-~/.local/state}/pergola/layout.json` (document `version: 2`,
  shape-compatible with the layout files described in the research note).

Architecture: `pergola-core` (pure engine, no X11 — tree, geometry, camera,
commands, document), `pergola-x11` (EWMH enumeration, moves, hotkey grabs,
overlay), `pergola` (daemon + CLI + Unix-socket IPC). See
[docs/DESIGN.md](docs/DESIGN.md).

## Known limitations

* **X11 only.** No Wayland-native support; XWayland clients only.
* Verified inside **xfwm4** (XFCE) and **Openbox**, live and in isolation; any
  EWMH window manager should work, but only those two have been measured.
* Windows outside the framed view are moved off-screen. Some WMs (xfwm4) clamp
  them to keep a sliver reachable, so a docked tile can peek at the screen edge.
* Windows that set size hints (terminals with a character grid) may not fill a
  tile exactly; the WM honours their hints. Under Openbox an `xterm` with 6x13
  resize increments lands 11 px short of a 900 px tile; xfwm4 rounds differently.
  This is the client's hint, not the layout: `pergola rects` still reports the
  tile.
* Dialogs and transient windows are treated as ordinary panels when docked.
* Tab groups are modelled (`views.len() > 1`) but there is no drag-to-tab UI yet.
* The overlay is drawn with 2–3 px override-redirect rectangles inside the gaps
  and around the viewport; there is no dimming without a compositor.
* Only one instance per display (one Unix socket).

## Development

```bash
cargo test               # 46 unit tests: geometry, tree, camera, commands, document
scripts/e2e.sh           # 76 assertions against Xvfb + xfwm4 + real xterms
scripts/health.sh        # soak check of a running daemon (cpu, layout, drift, orphans)
scripts/health.sh --e2e  # the above plus the regression suite
cargo build --release
```

`install.sh` also installs `pergola-health`, the same soak check under a stable name, so a
scheduler or a timer can run it unattended and act on the exit code. `BACKLOG.md` is the
development queue: every item names what is done when.

The end-to-end script starts its own X server, window manager and windows, so it
does not touch your session. It asserts geometry as the X server reports it, not
just the daemon's own bookkeeping.

Versioning: [docs/VERSIONING.md](docs/VERSIONING.md). Current version is
`0.2.1-dev` — pre-stable, no compatibility promises.

## License

MIT © SyntheticPrometheus. "TrellisWM" is a trademark of its owner and is used
here only to describe the model Pergola reimplements. No code from TrellisWM or
from the clean-room study is included. See [NOTICE](NOTICE).
