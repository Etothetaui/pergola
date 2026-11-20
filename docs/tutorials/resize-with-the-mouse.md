# Resize with the mouse

The gap between two tiles is not just space: it is the handle. Drag it and the split resizes, both
neighbours moving, neither falling below `minimum_share`.

**Starting state.** Two or more tiles in one split, `gap` at its default 6 px, the daemon running.
The overlay is not required — the grab strip exists whether or not split lines are drawn.

## Steps

**1. Find the strip.** Move the pointer into the gap between two tiles without pressing anything: a
3 px line appears down the long axis of the band, and `pergola status` grows a `· hint hover` field.
That is the only affordance — a 6 px gap is otherwise invisible, so the hint says "this is
draggable" before you commit.

**2. Drag it.** Press the primary button on the strip and move along the split's short axis. You
should see the boundary follow the pointer, both neighbours resize; the pointer keeps the grab even though the
boundary moves out from under it. Release to finish.

**3. Watch the floor.** Keep dragging past the point where one side would vanish: it stops at
`minimum_share` (0.05 of the split by default) and keeps stopping there for the rest of the drag. To
change the floor, set `minimum_share` in the config file and `pergola reload`.

**4. Undo it if you overshot.**
```bash
pergola undo
```
A gap drag is one layout change like any command. Each motion event re-snapshots the tree, so a long
drag cannot accumulate drift: the last snapshot is the layout you get back.

**5. Drag a tile, not the gap.** A docked tile dragged off its own rectangle (travelling at least 40
px and ending more than 14 px from where it was recorded) floats where you drop it — the mouse route
back to the desktop layer, no command needed. Released near a different edge instead, it re-docks
there in the same gesture.

## Verify

```bash
pergola rects
```
Run it before and after a drag: the two panels of that split change `w` (or `h`) by equal and
opposite amounts, the right-hand `x` follows the boundary, and the row still ends at the work area's
edge. `pergola status` carries `· hint hover` while the pointer is over a strip and `· hint drop`
while a window is dragged near an edge; when the pointer leaves both, the field is gone — cleaned
up, not left on screen.

**Where this bites.** * With `gap 0` there is no grab strip at all: use [resize with the
keyboard](resize-with-the-keyboard.md) — `pergola ratio` — or raise the gap first.
* The hint needs the X SHAPE extension. Without it the line is simply not drawn
and the pointer still works, so a missing hint is cosmetic.
* A drag is cancelled by the next command that arrives mid-gesture (a hotkey,
another terminal); the layout keeps the last committed tree.
* The strip is as wide as `gap`, so a 2 px gap is a 2 px target: `gap` is a
comfort setting as much as a visual one.
