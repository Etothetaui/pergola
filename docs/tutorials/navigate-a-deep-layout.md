# Navigate a deep layout

The camera is a framed node plus back/forward history: framing a node maps it onto the work area and
everything else moves with it — a continuous zoom, not a workspace switch.

**Starting state.** The layout from [build a three-window layout](build-a-three-window-layout.md) —
three tiles, the right half holding a top/bottom pair, `depth 2` — with `pergola status` showing
`framed split-gen1`, the root a fresh layout starts on.

## Steps

**1. Frame the focused window.**
```bash
pergola frame-window
```
You should see it fill the work area, the windows behind it move off screen, and the command prints `frame-window → framed p-… (depth 2)`.
Run it again on the same window and it frames the root instead: framing is a toggle.

**2. Step out one level.**
```bash
pergola zoom-out
```
You should see the tile shrink to its real share and the whole split containing it comes into view: `zoom-out`
frames the parent of whatever is framed, falling back to the root when there is no parent.

**3. Step one level back in.**
```bash
pergola zoom-in
```
`zoom-in` frames the first non-desktop child of the framed node, so repeated presses walk down the
tree one split at a time from wherever the camera is.

**4. Walk the camera history.**
```bash
pergola back
pergola forward
```
Each line names the node it framed. History is a browser stack, so `back` twice then `forward` once
lands on the previous framing, not on a guess.

**5. Move focus between spaces.**
```bash
pergola neighbour left
```
Left/right move between the columns of the split that contains the focused window; `up`/`down` move
inside a column. The camera follows the focus, which is why `framed` changes with it.

**6. See everything again.**
```bash
pergola everything
```
You should see all three tiles back in view and `framed` returns to the root split's id.

## Verify

```bash
pergola status
```
`framed <id>` is the id you asked for — `split-gen1` after `everything`, `p-…` after `frame-window`
— while `depth N` never changes: the camera does not edit the tree. `pergola rects` confirms it
geometrically, the framed panel filling the work area.

**Where this bites.** The chords are the *plan*: `Ctrl+Alt+Return` frames, `Ctrl+Alt+Up`/`Down`
zoom, `Ctrl+Alt+[`/`]` walk history. A desktop that owns one of them (xfwm4 does) gets the `Super+…`
fallback, and `pergola keys --check` prints what actually fired. Framing is non-uniform — the framed
node is mapped exactly onto the work area, so a tall window is stretched to that aspect and
everything else moves by the same scales. `pergola everything` undoes every camera move in one
command.
