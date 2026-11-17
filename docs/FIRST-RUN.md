# Your first five minutes

Pergola is a *layer* on top of your existing desktop. It does not replace your
window manager, change your wallpaper or take over your session — it adds
nesting tiles and a camera on top of the windows you already have. If that is
the first thing you have read, this page is the shortest path from "installed"
to "I can see what this does".

If you have not installed yet, do that first (`README.md` → Install, or
`docs/SETUP.md`); this page assumes `pergola` is on your `PATH` and the daemon
is running.

## Check it is alive

```bash
pergola doctor    # display, EWMH client list, running WM, version
pergola status    # what the daemon thinks: screen size, panels, depth, gap
```

`doctor` should name your display and your window manager. `status` should
print one line starting with `pergola 0.5.0-dev`. If either says "daemon not
running", start it with `pergola daemon` in a terminal (or
`systemctl --user restart pergola`), then try again.

## The five-minute walkthrough

Open two terminals. In the first, run `pergola dock left`. It will take the
whole work area at first. Switch to the second terminal with your desktop's
taskbar or Alt+Tab, then run `pergola dock right` there. You should now see two
tiles side by side.

```bash
pergola dock left     # first terminal
pergola dock right    # second terminal, after switching to it
```

To see the nesting that makes Pergola different, keep the second tile focused
while you open a third terminal. Run `pergola dock down` in the new terminal.
The right tile becomes a top/bottom pair inside the left/right split. Pergola
uses the last focused tile as the target, so the focus order matters here.

The next commands show the camera and the saved layout:

```bash
pergola frame-window  # frame the focused window (run it again to go back)
pergola zoom-out      # step one level out — the whole tree comes into view
pergola everything    # zoom all the way out
pergola tree          # the layout document, as JSON
```

That is the whole idea: windows nest inside windows (a split inside a split),
and the camera decides which part of the nest fills the screen. Everything else
is commands over that model.

## Do it with the keyboard or the mouse instead

The terminal is only one way in. With a window focused:

* `Ctrl+Alt+Shift+←` / `→` / `↑` / `↓` — dock the focused window to that edge.
* `Ctrl+Alt+Shift+W` — float the focused window back to the desktop.
* `Ctrl+Alt+M` — the action palette, a menu of the same actions by name.
* Drag a floating window to a screen edge and let go within 14 px — it docks.
* Drag the gap between two tiles — it resizes the split.
* Drag a tile away from its edge — it floats where you dropped it.
* `Ctrl+Alt+Page_Down` / `Page_Up` — switch between windows stacked in one tile
  (a tab group); `pergola tab` folds the focused window into its tile as a tab.
* `pergola window-tag work` — put the focused window on a named workspace;
  `pergola tag-view work` switches to it and `pergola tag-view all` shows every
  window again. Tags have no default chords: pick one with
  `pergola bind "tag-view work" Super+w`.

If the desktop already owns a chord (`xfwm4` owns `Ctrl+Alt+arrows`), Pergola
falls back to `Super` for that command and says so at startup. `pergola keys
--check` prints the chord actually bound for each command. To pick your own
chord instead, run `pergola bind "gap 12" Super+9` — it writes the config,
reloads the daemon and tells you if the chord is taken. `pergola bind "gap 12"`
with no chord shows what it is bound to; `--default` restores the default. For
several changes at once, edit the `bindings` map in `config.json` and run
`pergola reload`; see [Configuration](SETUP.md#remapping-keys).

## The words on the screen

The first line `pergola status` prints is dense. Here is what each field means:

| Field | Meaning |
|---|---|
| `pergola 0.5.0-dev` | the version of the running daemon |
| `1600x874` | the work area the daemon is tiling into (the screen minus panels/docks) |
| `N panels` | how many windows are currently tiled in the tree |
| `N floating` | how many windows are on the desktop layer (not tiled) |
| `depth N` | how many levels deep the split tree nests |
| `framed <id>` | which node the camera is currently framed on (`split-gen1`, `p-…`, or `desktop`) |
| `gap N` | the pixel gap between tiles (and the width of the drag strip) |
| `overlay on/off` | whether split lines (and the dim layer, if a compositor can render it) are drawn |
| `active 0x… (p-…)` | the focused window and its panel id, or `unmanaged` |

## Glossary

| Term | What it means here |
|---|---|
| **layer** | Pergola runs *inside* your window manager, not instead of it. Your WM still owns decorations, constraints and the session; Pergola only moves windows and draws gaps. |
| **panel** | One tiled window. A panel is a leaf of the tree and owns a window id (`p-3176783`). |
| **seat** | The daemon's record of one window: which panel it occupies, its title and class. |
| **split** | A rectangle divided into two or more children along one axis (`x` or `y`), with a weight per child. A split may contain splits — that recursion is the "fractal" part. |
| **desktop** | A *space*, not the screen. Floating windows live inside whatever rectangle the desktop leaf currently occupies. Exactly one desktop leaf exists. |
| **camera** | A framed node id plus back/forward history and nine saved views. Framing a node maps it onto the screen and moves everything else with it — a continuous zoom, not a workspace switch. |
| **framed** | The node the camera is currently centred on. |
| **depth** | How many split levels the tree nests. |
| **gap** | Pixels between tiles; also the width of the strip you drag to resize a split. |
| **overlay** | Split lines drawn with 2–3 px rectangles in the gaps (off by default). When a compositor is running it also dims the tiles, so the lines stand out. |
| **dock** | Move a window into the tree against an edge of its space (tile it). |
| **float** | Return a window to the desktop layer (untile it). |
| **view** | A saved camera position (`save-view 1`, `view 1`). Confusingly, a panel's *views* are also its tabs; the command name `tab-select` is used when the tab meaning is meant. |
| **tab** | A second window stacked in the same tile as the first. Only the selected tab is visible; the others stay mapped behind it. `pergola tab`, `tab-select`, `tab-next`, `tab-prev`. |
| **tag** | A named workspace: a set of views shown together. `pergola window-tag work` moves a window onto one, `tag-view work` shows only that tag's windows (the rest are moved off the work area and keep their tiles), `tag-view all` shows everything again. Not the same as a *view*: a tag is a workspace, a view is a saved camera position. |
| **layout document** | The JSON tree + camera + window fingerprints persisted to `~/.local/state/pergola/layout.json`. |
| **work area** | The screen minus panels and docks, from `_NET_WORKAREA`. Pergola tiles inside it. |
| **EWMH** | The X11 standard for window-manager hints and client lists. Pergola uses `_NET_MOVERESIZE_WINDOW` to move windows. |
| **X11 only** | Wayland clients cannot be moved by an external process; Pergola needs X11 (or XWayland for X11 clients). |

## If nothing happens

| Symptom | What to do |
|---|---|
| A hotkey does nothing | `pergola keys --check` — the desktop may own the chord; Pergola falls back to `Super` and reports it at startup. |
| `Ctrl+Alt+M` does nothing | No menu program. Run `pergola palette` in a terminal to see the error; install `rofi` (recommended) or set `PERGOLA_MENU=/path/to/menu`. |
| `pergola dock left` says "not managed by pergola" | The focused window is a dialog or transient (`WM_TRANSIENT_FOR`); those are not tiled by design. |
| A window will not move | The WM ignores `_NET_MOVERESIZE_WINDOW` (common for tiling WMs that lay themselves out). Only Xfwm4 and Openbox are measured. |
| A dropped window landed somewhere nested | Edge drops dock against the last focused space by design — drop again on the edge you want, or use `Ctrl+Alt+Shift+arrows` after focusing the target tile. |
| "a pergola daemon is already running" | Expected: one instance per display. Stop the old one with `pergola stop` first. |
| `pergola tab-next` says "not a tab group" | The focused window's tile holds only one window. Fold another in first with `pergola tab --window <id>`, or focus a window in a group that already has more than one. |

More: `docs/SETUP.md` → Troubleshooting, `docs/UX-NOTES.md` for the known
human-interface pain points (with severity and proposed fixes), and
`docs/HISTORY-LESSONS.md` for the failure classes Pergola does and does not
handle.
