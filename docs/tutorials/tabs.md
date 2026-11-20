# Tabs

A tab is a *window inside one panel*: the tile keeps its rectangle, holds several windows, and shows
one of them. It is not a browser tab bar and not a workspace — those are [saved
views](saved-views.md) and [numbered workspaces](numbered-workspaces.md).

**Starting state.** One tile docked and one more window floating, with `pergola status` showing `1 panels, 1 floating · depth 1`.
That tile is the last focused space, which is where a fold lands.

## Steps

**1. Fold the floating window into the tile.** Focus the floating window, then:
```bash
pergola tab
```
You should see it leave the desktop and join the tile; the reply reads `tabbed p-… into p-… (2 views)`. The
target is the last focused panel — the window in front before this one decides where it goes. Choose
explicitly with `--target`: `pergola tab --target p-…`.

**2. Move between them.**
```bash
pergola tab-next
pergola tab-prev
```
Each replies `showing p-… in p-…`: the same tile shows that window inside the same tile. The chords are
`Ctrl+Alt+Page_Down` / `Ctrl+Alt+Page_Up`, or `Super+…` when the desktop owns the primary.

**3. Select a specific one.**
```bash
pergola list
```
Lists every window with its state and id (`docked`, `floating`, `tab`, `parked`, `hidden`). The
unselected views of the group read `tab`, not `docked`: they sit at the tile but are covered. Pick
one:
```bash
pergola tab-select 0x…  # the id you just read
```
`tab-select` takes a window id or a seat id (a panel's `views` in `pergola tree`).

**4. See what the panel holds.**
```bash
pergola status
```
The line grows `· 1 tabs`: the count is *extra* views beyond the first, so two windows in one panel
read as one tab.

## Verify

```bash
pergola tree
```
Find the panel node: `views` has two entries, `selected` names the visible one, `tabbed` is `true`.
A single-window panel has a one-element `views` and no `tabbed` key at all. `pergola list` shows
exactly one line reading `tab`.

**Where this bites.** There is no tab bar, no drag-to-tab and no mouse tab selection — deliberately,
not by omission: the tile's rectangle is the only handle the pointer has, and a bar would live
inside a window the window manager still decorates. Folding and switching are commands and chords
only. `tab-next` on a single-window panel answers "no tab group to cycle", so a keypress is never
silently ignored. A window that closes is dropped from the group by the reconciliation pass and the
remaining views stay.
