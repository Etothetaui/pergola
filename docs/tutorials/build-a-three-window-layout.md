# Build a three-window layout

Pergola tiles by docking a window against an edge of *its own space*: three docks give three tiles,
and the third one nests inside the second.

**Starting state.** `pergola` on `PATH`, the daemon running, three terminal windows open and nothing
tiled. Untouched config: `gap` 6 px, `auto_dock` off, `edge_dock` true.

## Steps

**1. Confirm the daemon.**
```bash
pergola status
```
You should see one line starting `pergola 0.5.0-dev`, then the work area, `0 panels, 3 floating`,
`depth 0`, `framed desktop`. Nothing on screen changes; if it says the daemon is not running, start
one with `pergola daemon`.

**2. Dock the first window left.**
```bash
pergola dock left
```
You should see it take the whole work area; it prints `docked p-… left of desktop (depth 1)`: with only the
desktop leaf to sit beside, "left of the desktop" is the entire space.

**3. Alt+Tab to the second terminal and dock it right.**
```bash
pergola dock right
```
You should see two tiles side by side, half the width each minus the 6 px gap, and `depth 1` — one split with two
panels.

**4. With the second tile focused, open a third terminal and run this in it.**
```bash
pergola dock down
```
You should see the right tile become a top/bottom pair. The new window is the one being docked; the target is the
last focused tile, so it lands *inside* the right half. The command reports `depth 2`.

**5. Read the tree back.**
```bash
pergola tree
```
`doc.root` is a `split` on axis `x` whose second child is another `split` on axis `y`: splits nest,
and that recursion is the whole model. Then:
```bash
pergola rects
```
Three lines of `panel-id x y w h`, one per panel, through the camera's framing.

**6. Or do steps 2–3 with the mouse.** Drag a floating window until the pointer is within 70 px of a
screen edge — a 3 px frame previews the landing rectangle — and release within 14 px of it. Twice
(left, then right) reproduces step 3. A drop resolves against the last focused space, exactly as
`dock` does.

## Verify

```bash
pergola status
```
The line reads `3 panels, 0 floating · depth 2`, and `framed <id>` names the panel `pergola rects`
lists first. For the nesting rather than the count, `pergola tree` must show a `split` inside a
`split`.

**Where this bites.** `dock` needs a focused window and refuses dialogs and transients
(`WM_TRANSIENT_FOR`), which are never tiled by design. To aim at a space that is not the last
focused one: `pergola dock down --target p-…`.
