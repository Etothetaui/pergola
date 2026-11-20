# User guide

Pergola turns the windows you already have into a nest, and gives you a camera to
look at one part of the nest at a time. This page is the illustrated tour: model,
sequence of docks, camera, keyboard, mouse, defaults, recovery. It is a guide, not
a reference — the links at the bottom are the reference, starting with
[Your first five minutes](FIRST-RUN.md) and [Setup](SETUP.md).

## A layer, not a window manager

Pergola runs *inside* your window manager. It never selects `SubstructureRedirect`
on the root window, so the display still belongs to whatever you already use:
decorations, focus policy, taskbar, workspaces, session and all. Pergola only asks
the WM to move windows (`_NET_MOVERESIZE_WINDOW`, the same request `wmctrl` makes)
and draws gaps.

The practical difference from a traditional tiling window manager:

| | Traditional tiling WM | Pergola |
|---|---|---|
| Who owns the display | the tiling WM *is* the WM | your existing WM keeps it |
| What it decides | geometry, decorations, policy | geometry and gaps only |
| Wayland | often its own compositor | X11 clients only |
| Support here | — | xfwm4 and Openbox measured |

Because it holds no redirect, it cannot support a WM that lays windows out itself
and ignores EWMH moves; tiling WMs such as i3 or awesome are *expected* to work,
but only xfwm4 and Openbox are measured. See [Compatibility](COMPATIBILITY.md) and
`pergola doctor`, which names the WM it found.

## The model: panels, splits, the desktop

Five words carry the whole model:

* **panel** — one tiled window, a leaf of the tree, with its own id (`p-3176783`);
* **split** — a rectangle cut into children along one axis (`x` side by side, `y`
  stacked), each with a weight; a split may contain splits;
* **desktop** — a *space*, not the screen: the leaf floating windows live in.
  Exactly one exists; its share is `0` by default, so it costs no pixels until you
  frame it;
* **camera** — which node is framed, i.e. mapped onto the screen;
* **gap** — the pixels between tiles, and the strip you can drag.

The whole layout is that one tree: `pergola tree` prints it as JSON, `pergola
status` prints the summary line (`N panels`, `depth N`, `framed …`).

## Nesting, one dock at a time

Each step is one command on one focused window, with the tree afterwards.

**1. One window docks left.** `Ctrl+Alt+Shift+←` (`pergola dock left`). It takes
the work area; the desktop leaf is still in the tree at share `0`, so it occupies
nothing until you frame it.

```text
 ┌──────────────────────────────┐        root ── split(x)
 │  ┌────────────────────────┐  │           ├─ p-editor   1.00
 │  │ p-editor               │  │           └─ desktop    0.00
 │  └────────────────────────┘  │
 └──────────────────────────────┘
```

**2. A second docks alongside.** Focus the second window, then
`Ctrl+Alt+Shift+→` (`pergola dock right`). The edge is taken from the *space*
around the focused window, so the tree gains a sibling rather than a new tree.

```text
 ┌───────────────┬──────────────┐        root ── split(x)
 │ p-editor      │ p-browser    │           ├─ p-editor   0.50
 │               │              │           ├─ p-browser  0.50
 │               │              │           └─ desktop    0.00
 └───────────────┴──────────────┘
```

**3. A third docks below the second.** Focus `p-browser`, then
`Ctrl+Alt+Shift+↓` (`pergola dock down`). The right-hand rectangle is what gets
split — the fractal step: a split inside a split, `depth 2`.

```text
 ┌───────────────┬──────────────┐        root ── split(x)
 │               │ p-browser    │           ├─ p-editor   0.50
 │ p-editor      ├──────────────┤           ├─ split(y)   0.50
 │               │ p-terminal   │           │   ├─ p-browser  0.50
 │               │              │           │   └─ p-terminal 0.50
 │               │              │           └─ desktop    0.00
 └───────────────┴──────────────┘
```

**4. A fourth nests deeper.** Focus `p-terminal` and dock a new window left of it
(`pergola dock left`). Nothing reflows: its rectangle is subdivided, `depth 3`, and
every other tile keeps its geometry.

```text
 ┌───────────────┬──────────────┐        root ── split(x)
 │               │ p-browser    │           ├─ p-editor   0.50
 │               ├───────┬──────┤           ├─ split(y)   0.50
 │ p-editor      │p-notes│p-term│           │   ├─ p-browser 0.50
 │               │       │      │           │   └─ split(x)  0.50
 │               │       │      │           │       ├─ p-notes    0.50
 │               │       │      │           │       └─ p-terminal 0.50
 │               │       │      │           └─ desktop    0.00
 └───────────────┴───────┴──────┘
```

Docking resolves against the **last focused space**, which is what makes step 3 land
in the right column rather than across the screen. If a drop surprises you, focus
the tile you meant and dock again.

## The camera

The screen shows one node: the framed one. Framing is a transform, not a workspace
switch — the framed node is mapped onto the screen and everything else moves with
it. **Framing never restructures the tree**: no split is created or collapsed, no
weight changes.

```text
 frame-window              zoom-out one level       everything
 ┌───────────────┐         ┌─────────────┬───────┐  ┌──────┬──────┐
 │  p-notes      │         │  p-notes    │p-term │  │p-edit│p-brow│
 ├───────────────┤         │             │       │  │      │──────│
 │  p-terminal   │         └─────────────┴───────┘  └──────┴──────┘
 └───────────────┘         framed: split(x)         framed: root
 framed: the split inside p-terminal's column
```

| What you want | Keyboard | CLI |
|---|---|---|
| Frame the focused window | `Ctrl+Alt+Return` | `pergola frame-window` |
| Out / in one level | `Ctrl+Alt+↑` / `Ctrl+Alt+↓` | `pergola zoom-out`, `pergola zoom-in` |
| Whole tree | `Ctrl+Alt+Space` | `pergola everything` |
| Back / forward through camera history | `Ctrl+Alt+[` / `Ctrl+Alt+]` | `pergola back`, `pergola forward` |
| Frame or show/hide the desktop layer | `Ctrl+Alt+d` | `pergola frame-desktop`, `pergola toggle-desktop` |
| Save a camera position, return to it | — | `pergola save-view 1`, `pergola view 1` |

Zooming while framed means "frame my parent", so repeated `zoom-out` climbs to the
root and `pergola everything` jumps there.

## Keyboard: navigation and modification

Navigation moves focus without touching the tree; modification moves windows or
boundaries. `Ctrl+Alt+←` / `→` (`neighbour left` / `neighbour right`) walk between
columns of the split that contains the focused window; `neighbour up` / `down`
exist too, with no default chord. `Ctrl+Alt+,` and `Ctrl+Alt+.` shift the boundary
next to the focused panel by 5% of its split (`ratio -0.05`, `ratio +0.05`).

Which chord actually fires depends on your desktop. Pergola tries `Ctrl+Alt+…`
first and falls back to `Super+…` per command when the host WM already owns that
chord — xfwm4 takes `Ctrl+Alt+arrows` and `Ctrl+Alt+D`, so the camera keys land on
`Super` there. `pergola keys --check` prints what this session really grabbed:
that output is authoritative, the tables are the plan. `Ctrl+Alt+M`
(`pergola palette`) opens the same actions by human name and needs `rofi`,
`zenity`, `yad` or `dmenu`. The full plan is in [Keyboard bindings](KEYBINDINGS.md).

## Mouse: the same operations

No command line is needed for the common moves:

* **Dock** — drag a floating window to a screen edge and release within 14 px. A
  3 px frame shows the rectangle you will get while the pointer is within 70 px of
  an edge, so the drop is visible before you let go. In a nested layout a drop
  joins the last focused space, the same rule the keyboard uses.
* **Resize a split** — drag the gap between two tiles. It is a grab strip the
  width of `gap` (6 px by default) with a resize cursor, both neighbours move, and
  no boundary can be pushed below `minimum_share`. With `gap` at 0 there is no
  strip; use `pergola ratio`.
* **Float** — drag a docked tile away from its edge and release it elsewhere: it
  floats there. Dropping it on a *different* edge re-docks it in one gesture.
* **Palette** — `Ctrl+Alt+M`, or **Pergola Actions** in your application menu, is
  mouse-clickable and type-to-filter.

Hovering a gap lights up a 3 px line down its long axis, so a 6 px strip is
findable. That hint and the drop preview need the SHAPE extension; without it they
are not drawn and the pointer still works. Details: [Mouse](MOUSE.md). There is
**no drag-to-tab and no tab bar** — stacking windows in one tile is a keyboard and
command feature (`Ctrl+Alt+Page_Down`, `pergola tab`).

## Both at once

The engine does not care whether a command came from a chord, a socket or a drag:
the keyboard sends the same argv the CLI sends, a drag re-derives the tree from a
snapshot. The two mix freely and compose:

* drag a boundary with the mouse while framed, then step out with `Ctrl+Alt+↑` to
  see the change in context;
* dock three windows with the pointer, then `Ctrl+Alt+,` on the one you use most;
* frame a tile, dock into it with an edge drop, and `Ctrl+Alt+z` (`pergola undo`,
  which does not distinguish sources) steps back.

The pointer thresholds (14 px snap, 70 px preview, 40 px travel) are measured
facts, not knobs; what you *can* change is `gap`, `edge_dock`, `minimum_share` and
`auto_dock`.

## The action / key / mouse / command triple

| Action | Keyboard | Mouse | CLI |
|---|---|---|---|
| Dock left | `Ctrl+Alt+Shift+←` | drag to the left edge, release within 14 px | `pergola dock left` |
| Dock right | `Ctrl+Alt+Shift+→` | drag to the right edge | `pergola dock right` |
| Dock up / down | `Ctrl+Alt+Shift+↑` / `↓` | drag to the top / bottom edge | `pergola dock up`, `pergola dock down` |
| Resize a split | `Ctrl+Alt+,` / `Ctrl+Alt+.` | drag the gap strip | `pergola ratio -0.05`, `pergola ratio +0.05` |
| Float a window | `Ctrl+Alt+Shift+w` | drag the tile off its edge, release | `pergola float` |
| Frame / zoom | `Ctrl+Alt+Return`, `Ctrl+Alt+↓`, `Ctrl+Alt+↑` | — | `pergola frame-window`, `pergola zoom-in`, `pergola zoom-out` |
| Undo the last layout change | `Ctrl+Alt+z` | — | `pergola undo` |
| Arrange / float everything | `Ctrl+Alt+a` / `Ctrl+Alt+f` | — | `pergola arrange`, `pergola float-all` |

Chords listed are the `Ctrl+Alt` candidates; on a desktop that owns them the same
command sits on `Super+…` (`Ctrl+Alt+z` → `Super+z`). Check with
`pergola keys --check`.

## Changing the defaults

Everything optional lives in one file, `~/.config/pergola/config.json`, and every
key has a working default, so an empty file is a valid configuration.

```bash
pergola config            # every key, its value, and where it came from
pergola config --check    # exits non-zero on an unknown key or a bad value
pergola settings          # the same keys as a menu
pergola bind "gap 12" Super+9   # one command, one chord, validated and reloaded
pergola unbind "zoom-in"        # remove a chord
pergola reload                  # re-read the file; an invalid file is refused
```

`pergola bind` and `pergola settings` write the file for you and reload the daemon;
edits by hand need `pergola reload`. A rejected file leaves the previous settings
in force, which is why nothing goes dark when you get a value wrong. The keys and
ranges — and which ones need a daemon restart rather than a reload — are in
[Configuration](CONFIGURATION.md); the chord grammar (modifier names, key names,
what to do when a symbol is missing from your layout) and the default plan are in
[Keyboard bindings](KEYBINDINGS.md).

## Recovering from mistakes

Nothing here loses windows: the tree is a model, and the windows are the desktop's.

| Situation | Do this |
|---|---|
| A dock landed somewhere wrong | `Ctrl+Alt+z` — `pergola undo`, 10 layout changes deep |
| You zoomed into a corner | `pergola everything` — the camera back to the whole tree |
| You want the flat desktop back | `Ctrl+Alt+f` — `pergola float-all`, every window to the desktop layer |
| A window will not move | the host WM ignores EWMH moves; `pergola doctor` says which WM |
| A hotkey does nothing | `pergola keys --check`; the desktop may own that chord |
| The daemon misbehaves | `pergola stop` — windows stay put; `pergola daemon` starts it again |

`pergola undo` covers layout changes only, not configuration. `pergola float-all`
*is* a layout change, so `pergola save` makes it permanent and `pergola undo` takes
it back — the halves are deliberately separate. To see what the daemon believes,
`pergola tree`, `pergola list`, `pergola rects` and `pergola status` print it.

## Where to go next

1. [Your first five minutes](FIRST-RUN.md) — walkthrough and glossary.
2. [Setup](SETUP.md) — install, upgrade, uninstall, config file, troubleshooting.
3. [Keyboard bindings](KEYBINDINGS.md) — the plan, chord grammar, remapping.
4. [Mouse](MOUSE.md) — drag thresholds, hints, what the pointer can do.
5. [Configuration](CONFIGURATION.md) — every key, default, range, reload rule.
6. [Commands](COMMANDS.md) — every command, its arguments and its output.
7. [Compatibility](COMPATIBILITY.md) — measured WMs, Wayland, "unmeasured".
8. [Design](DESIGN.md) — why a layer, why the camera is a transform, why the desktop
   is a space; [UX notes](UX-NOTES.md) for the known rough edges.
