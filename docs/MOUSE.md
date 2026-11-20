# Mouse

Pergola adds one pointer surface of its own — the strip between two tiles — and
otherwise watches the window drag your desktop already performs. It never takes
`SubstructureRedirect`, so the window manager keeps the titlebar, the move gesture
and the pointer grab; Pergola reads the resulting `ConfigureNotify` events and,
when a drag settles against an edge, docks the window.

* **No modifier is required by Pergola.** Nothing here needs `Alt`, `Ctrl` or
  `Shift`. The button that starts a *window* move is your desktop's: button 1 on
  the titlebar on xfwm4 and Openbox, `Alt`+drag where configured.
* Only **button 1** drags a split boundary. Any other button pressed on a strip
  ends a gesture that is already running and does nothing itself.
* Every cue is override-redirect with an empty input region, so it never takes the
  click. Without SHAPE no cue is drawn — strips and docking still work.

## The gestures

| Gesture | Pointer starts on | Button | What you see | On release | Affected by |
|---|---|---|---|---|---|
| Hover a gap | the strip between two tiles | none | a 3 px line down the strip's long axis, click-through | nothing | `gap` |
| Drag a gap | the same strip, press and move | 1 | the line, then both neighbours resize live under the pointer | weights saved, strips rebuilt, one undo entry | `gap`, `minimum_share` |
| Drag a window to a screen edge | the window's titlebar / your desktop's move gesture | 1 on the frame | a 3 px frame around the rectangle the drop will produce, while the pointer is within 70 px of a work-area edge | docks against the nearest edge the frame is within 14 px of | `edge_dock`, `gap` |
| Drag a docked tile to another edge | same | same | same frame | re-docks there in one gesture, no float step in between | `edge_dock`, `gap` |
| Drag a docked tile away from its edge | same | same | no frame unless the pointer nears an edge | floats where it was dropped | `edge_dock`, `gap` |
| Drop a floating window mid-screen | same | same | no frame | stays floating where the desktop left it, still outside the tree | — |

The frame comes from the same insertion call the dock uses, after taking the
window's seat out of the tree, so it is the real target, not a half-screen guess —
see [Nested layouts](#nested-layouts-the-drop-is-not-at-the-root).

## The numbers

| Figure | What it measures |
|---|---|
| **6 px** | the gap strip with the shipped `gap` of `6.0`: exactly the gap between the two child rectangles, that wide and as long as their parent on the other axis, with a resize cursor |
| **3 px** | the hover line, centred in the strip, and each of the four bars of the drop frame |
| **14 px** | how far the dropped window's *frame* (client plus decorations) may be from a work-area edge and still dock (`EDGE_SNAP_PX`); also the tolerance for "still where the gesture left it", so a window that drifted further is not docked from that gesture |
| **70 px** | how far the *pointer* may be from an edge for the drop frame to appear (`DROP_EDGE_PX`). A look-ahead only: the dock is decided by the frame at release, so dropping just outside 70 px can still dock and a window inside 70 px may be released too far out |
| **40 px** | `DRAG_TRAVEL_PX`: a desktop that reports a drag as one event must have moved the window this far for it to count as a drag, not a placement |
| **0.05** | `minimum_share`, the floor a gap drag stops at |

`overlay` and `dim` change none of this: they draw the tree's lines (2 px, or 4 px
with no dim layer), while the hover line and drop frame are a separate cue layer
that appears whether the overlay is on or off.

## Dragging a split boundary

Press on a strip and the pointer is grabbed implicitly for the rest of the
gesture, so the boundary keeps following you even though it is the thing moving
under the pointer. Each motion is resolved against the tree snapshot taken at
press, so weights are absolute rather than accumulated: a slow drag and a fast
one end in the same place.

On release the strips are rebuilt at the new boundary positions and the layout is
saved; `pergola undo` walks the change back. Hitting the `minimum_share` clamp is
not an error — the motion stops changing the tree, and the only message is on
stderr under `PERGOLA_DEBUG`. The gesture also ends if a command arrives while the
button is down (a hotkey, an IPC client): the in-flight layout is committed first,
because the next motion would otherwise reinstate the snapshot and silently undo
the command. A reconcile pass, or the dragged window disappearing, does the same.

**`gap` 0 removes the surface.** A zero-width strip is not created, so there is
nothing to hover and nothing to press: change the split with `pergola ratio 0.3`
(or the palette's half/third rows), or raise the gap again with `pergola gap 8`.

## Dropping a window on an edge

A window counts as dragged when the desktop produced at least three configure
events for it, or one event that moved it 40 px or more. The daemon then waits
for 300 ms of quiet, checks the window is still within 14 px of where the gesture
left it, and only then looks for an edge.

* A **floating** window released within 14 px of an edge docks there. A corner
  goes to the nearer edge; an exact tie resolves left, right, up, down.
* A **docked** tile that left its own rectangle and settles near a different edge
  re-docks there in one step. Dropping it back on its own edge floats it, which
  is how "pull it off the edge" works.
* A **docked** tile dropped away from every edge floats where it was dropped. The
  rectangle is converted back through the camera into the desktop leaf, so a drop
  while zoomed is stored in world space.
* A window that moved since the gesture (a layout apply overtook a pending drag)
  has its watch discarded: docking from the stale rectangle would move a window
  the user never dropped there. Fullscreen windows and dialogs
  (`WM_TRANSIENT_FOR`) are ignored — their geometry belongs to the window manager
  or to their parent.

One gesture pushes at most one undo entry, taken before the first window is docked
or floated, and dropped again if the gesture changed nothing.

## Nested layouts: the drop is not at the root

The docking target is chosen exactly as `pergola dock <direction>` chooses it:
the last focused space. In a tree more than one level deep the seat you were last
working in gets the new neighbour, so a drop that looks like "the right third of
the screen" can land as the right half of the bottom-right quadrant.

Take `A | (B / C)`: A on the left, B above C on the right. You were last typing in
C, then drag a new window D to the right-hand edge. D does not become a
full-height third column; it becomes C's right-hand neighbour, so the bottom right
is split again and B keeps its full width.

Read the frame rather than the screen: the preview runs the same target selection
and detach-then-insert steps as the dock, so the nested answer is visible before
you let go. The drop joins whatever was last focused, so to sit beside a *specific*
tile, focus it first. For a root-level neighbour when the frame shows a nested one,
cancel and focus the tile you want to sit beside — or a floating window, whose
panel is not in the tree and whose focus therefore resolves the target to the root.

## Zoom and drag

* Strips are built in screen coordinates from the camera transform, and the
  pixel-to-weight denominator is taken on the axis the boundary moves on, so a gap
  drag works while zoomed and moves the amount it looks like it moves. The drop
  frame is transformed into the framed view too, so what it shows is what you will
  see after the drop. Edge detection stays in screen space: the 14 px and 70 px
  regions are at the physical edge of the work area, so a zoomed window has to be
  taken to the screen edge like any other.
* Known limitation, as recorded in [docs/UX-NOTES.md](UX-NOTES.md), severity 2: a
  drag-out while zoomed "converts screen coordinates as if they were world
  coordinates, so the floating window can land outside the desktop leaf". The
  current tree does run the drop rectangle through `untransform` and `into_world`,
  so the row may be stale; until it is re-measured, treat the drop frame as the
  prediction and `pergola undo` the float if it lands oddly.

## Host titlebars and native window-moving controls

Pergola does not decorate or move a window while you drag it; the desktop does and
Pergola reacts to the result. A window that draws its own header bar may leave
nothing for the window manager to grab — use the desktop's modifier drag
(`Alt`+drag on most setups) or the keyboard chords in
[docs/KEYBINDINGS.md](KEYBINDINGS.md). The feedback is only as good as the
desktop's configure stream: xfwm4 emits events while the pointer moves, so the
drop frame tracks the drag; Openbox coalesces the whole drag into one configure
after release, so there is no mid-drag frame but the drop is still judged (from
that event and the 40 px travel rule). The desktop also still owns the frame:
windows with size hints are fitted to their character grid first, so a tile can be
one cell narrower than its slot, and a parked window may be clamped into a sliver
at the edge — per-host numbers in [COMPATIBILITY.md](COMPATIBILITY.md).

## Implemented, and deliberately not

Implemented: the hover line on a gap, gap dragging with undo and clamping, edge
docking by drop, one-gesture re-dock, drag-out to float, click-through cues, and
taskbar clicks forwarded as activation requests. Not implemented, not hidden
behind a setting:

* **No drag-to-tab, no tab bar, no mouse tab selection** — tab groups are a
  command surface only: `pergola tab`, `pergola tab-select`, `pergola tab-next`,
  `pergola tab-prev`.
* **No preview with `gap` 0** (no strip to grab), **without SHAPE**, or with
  `edge_dock: false`.
* **No docking to another window's edge** — the targets are the four edges of the
  work area, not the edges of neighbouring windows.

## Failure modes

* **Nothing docked after a drag.** The gesture did not qualify: fewer than three
  configure events and under 40 px of travel, or the window drifted over 14 px
  from where the gesture ended it.
* **Docked, but the window did not move.** The desktop ignores
  `_NET_MOVERESIZE_WINDOW`; compare `pergola rects` with the screen
  ([COMPATIBILITY.md](COMPATIBILITY.md)).
* **No frame while dragging.** The pointer is further than 70 px from an edge,
  `edge_dock` is false, or the server has no SHAPE extension — `pergola doctor`
  reports the extension.
* **A gap drag stops before the tiles touch.** The clamp at `minimum_share`
  (`0.05`), silent by design.
* **`pergola undo` appears to do nothing once.** A stationary press on a strip
  still records the gesture, and the tree it restores is the one on screen.
