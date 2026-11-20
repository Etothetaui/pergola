# Resize with the keyboard

Two numbers control geometry: `ratio` shares a split's width between its children, `gap` is the
space between them. The keyboard nudges the first; the second is a session-wide setting.

**Starting state.** At least two tiles in one split (see [build a three-window
layout](build-a-three-window-layout.md)), one focused, and `pergola status` reporting `gap 6` and
`depth 1` or more. `minimum_share` is at its default `0.05`.

## Steps

**1. Grow the focused panel.**
```bash
pergola ratio +0.05
```
A leading sign means "nudge by this much": you should see the boundary next to the focused panel moves 5 % of
the split towards its sibling. The reply names the result and every weight in that split: `ratio 0.55 · split weights [0.55, 0.45]`.

**2. Shrink it again.**
```bash
pergola ratio -0.05
```
Same command either way, whatever the current size — you never have to know the present share to
move it.

**3. Set a share outright.**
```bash
pergola ratio 0.3
```
You should see the focused window take exactly 30 % of its split; the other children keep their proportions
relative to each other. This is the command for a split you have never touched, where a nudge would
be a guess.

**4. Use the chords instead.** `Ctrl+Alt+comma` is `ratio -0.05` and `Ctrl+Alt+period` is `ratio +0.05`.
Where the desktop owns those chords the daemon binds `Super+comma` / `Super+period` and says so at
startup.

**5. Push until it stops.** Repeat the grow chord on the same panel: the boundary halts when the
sibling reaches `minimum_share`, 0.05 of the split. Nothing can be squeezed away entirely. To allow
a different floor, set `minimum_share` in `~/.config/pergola/config.json` — greater than 0, at most
0.5 — and `pergola reload`.

**6. Change the gap.**
```bash
pergola gap 12
```
You should see every gap in the tree grow; the reply is `gap 12`. This changes the running session only: put
`"gap": 12` in the config file and reload to keep it, because `pergola gap` does not write the file.

## Verify

```bash
pergola rects
```
Take it before and after one `ratio` call: the focused panel's `w` moves by about 5 % of the split's
width and its neighbour takes the difference, so the row still fills the same area. A split inside a
split moves only the boundary at that level; the outer split's offsets stay put. `pergola status`
then reports `gap 12` on its line, which is also the proof that `pergola gap` applied.

**Where this bites.** `ratio` needs the focused panel to *have* a parent split: on a lone tile it
fails with "has no split to rebalance", and it needs a focused window, so run it from a terminal in
the tile you mean. With `gap 0` there is nothing for the pointer to grab, which is why this is also
the fallback for [resize with the mouse](resize-with-the-mouse.md).
