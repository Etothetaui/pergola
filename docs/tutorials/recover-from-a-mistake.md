# Recover from a mistake

Every layout change is a command over one document, and the daemon keeps the last ten of them. Recovery is one of four moves: undo the change, reset the camera, show the windows a workspace hid, or put every window back on the desktop layer.

**Starting state.** A layout you have just broken — a dock in the wrong place, a frame that lost the window you wanted, a workspace that seems to have swallowed everything. The daemon is running and `pergola status` prints its line.

## Steps

**1. Undo the last layout change.**
```bash
pergola undo
```
Reply: `undid the last layout change (N older to undo)`. A dock, float, arrange, gap change, ratio nudge or gap drag is one step each, and `undo` is never recorded, so pressing it twice walks back two changes instead of oscillating. The depth is 10; at the bottom it replies `nothing older left` and then `nothing to undo`.

**2. Lost in the camera, not the layout?**
```bash
pergola everything
```
You should see the whole tree come back into view with `framed` on the root — the move when a tile is on screen but the window you want is not. `pergola back` walks camera history instead, if history is what you want.

**3. A workspace hid the window.**
```bash
pergola tag-view all
```
You should see every view again: a tag moves the others below the work area rather than closing them. `pergola tag-view` lists the workspaces with `*` on the active one.

**4. Start the layout over.**
```bash
pergola float-all
```
Every docked window returns to the desktop layer (`floated N panels`): nothing closed, rectangles kept. From there `pergola arrange` tiles them into one balanced tree. `float-all` changes the running layout; `pergola save` writes it to the layout document the daemon starts from next time.

**5. A config edit that broke something.**
```bash
pergola reload
```
An invalid file is refused and the old settings and keys stay in force, so a bad edit cannot take a working session down. `pergola config --check` names each problem with its line.

**6. The daemon itself.**
```bash
pergola stop
pergola daemon
```
`stop` ends the daemon and leaves every window where it is — the layout document is already on disk. Start it again, or `systemctl --user restart pergola`.

## Verify

```bash
pergola status
```
The panel count, `depth`, `framed` and `gap` on that line are the recovered state, and `pergola tree` is the document it comes from. After `pergola stop` the same windows are still in place and `pergola status` reports no daemon — the proof that stopping is not destructive.

**Where this bites.** Undo does not touch `config.json`: `bindings`, `auto_dock` and `hotkeys` live in the file and are restored with `pergola bind … --default` or by editing it and reloading. It cannot un-close a window either — reconciliation drops seats that no longer exist.
