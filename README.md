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
permission prompt.

```bash
pergola doctor          # display, EWMH client list, running WM
pergola keys --check    # which chords are free on this desktop
```

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
| `overlay [on\|off\|toggle]` | draw the tree's boundaries |
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
* Windows outside the framed view are moved off-screen. Some WMs (xfwm4) clamp
  them to keep a sliver reachable, so a docked tile can peek at the screen edge.
* Windows that set size hints (terminals with a character grid) may not fill a
  tile exactly; the WM honours their hints.
* Dialogs and transient windows are treated as ordinary panels when docked.
* Tab groups are modelled (`views.len() > 1`) but there is no drag-to-tab UI yet.
* The overlay is drawn with 2–4 px override-redirect rectangles; there is no
  dimming without a compositor.
* Only one instance per display (one Unix socket).

## Development

```bash
cargo test            # 37 unit tests: geometry, tree, camera, commands, document
scripts/e2e.sh        # 78 assertions against Xvfb + xfwm4 + real xterms
cargo build --release
```

The end-to-end script starts its own X server, window manager and windows, so it
does not touch your session. It asserts geometry as the X server reports it, not
just the daemon's own bookkeeping.

Versioning: [docs/VERSIONING.md](docs/VERSIONING.md). Current version is
`0.0.0-dev` — pre-stable, no compatibility promises.

## License

MIT © SyntheticPrometheus. "TrellisWM" is a trademark of its owner and is used
here only to describe the model Pergola reimplements. No code from TrellisWM or
from the clean-room study is included. See [NOTICE](NOTICE).
