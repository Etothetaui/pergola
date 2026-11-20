# Saved views

A *view* is a camera position, not a workspace and not a tab set: one of nine slots that remembers
which node was framed, so one command puts the screen back the way it was. Slots are `1`–`9` and
live in the layout document, so they survive a restart.

**Starting state.** A layout worth returning to — at least `depth 1` from [build a three-window
layout](build-a-three-window-layout.md) — and `pergola status` showing `framed split-gen1`,
everything in view.

## Steps

**1. Choose a position and save it.** Frame a tile, then store the position:
```bash
pergola frame-window
pergola save-view 1
```
The reply echoes the framing that was stored: `save-view → framed p-… (depth 1)`. Nothing moves —
saving a view is not a camera move.

**2. Build a second position.**
```bash
pergola everything
pergola zoom-in
pergola save-view 2
```
Slot 1 is now one tile and slot 2 is the first split inside the tree.

**3. Jump between them.**
```bash
pergola view 1
pergola view 2
```
You should see the screen change: each reply names the node it framed (`view → framed p-… (depth 1)`) and the screen changes with it.
A view stores a node id rather than a rectangle, so a jump is exact even if the tree has been
resized since.

**4. Overwrite one.** `pergola save-view 2` again replaces slot 2. There is no delete: overwrite the
slot or stop using it.

## Verify

```bash
pergola tree
```
The document carries a `camera` object, and inside it `saved_views` maps the slots you used to the
node ids they framed:
```text
"camera": {
  "framed": "p-2a00003",
  "saved_views": { "1": "p-2a00003", "2": "split-gen1" },
```
`pergola status` gives the live half of the proof: `framed <id>` after `view 1` is the id slot 1
holds, and it changes again after `view 2`.

**Where this bites.** * Slots are camera positions, not layouts on disk: `view 1` restores what
fills the work area, not window sizes, tags or tabs.
* If the window a slot pointed at has closed, the camera normalises to the whole
tree rather than framing nothing — you get `pergola everything`, not an error.
* There are no default chords for `view` or `save-view`. Bind what you will
press: `pergola bind "view 1" Super+F1`, the same path as [remap shortcuts](remap-shortcuts.md).
* Slot numbers are not workspaces ([numbered workspaces](numbered-workspaces.md))
and not tabs ([tabs](tabs.md)).
