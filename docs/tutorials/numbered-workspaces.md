# Numbered workspaces

A tag is a named workspace: a set of views shown together. Switching lays out only that set; the
rest keep their place in the tree and their rectangles, so switching back is exact. A tag is not a
saved camera position ([saved views](saved-views.md)) and not a tab group ([tabs](tabs.md)).

**Starting state.** Two or three tiles docked from [build a three-window
layout](build-a-three-window-layout.md), no tags yet, and `pergola tag-view` printing `no tags — every view is shown`.
`tags` is still `false` in the config, the default.

## Steps

**1. Put a window on a workspace.** Focus a tiled window and run:
```bash
pergola window-tag work
```
The tag is created by the name. The reply is `moved p-… to tag work`; if the window left the visible
set it adds `(off screen until you switch to it)`. Pass `--window 0x…` for a window that is not
focused. A view belongs to at most one tag, so tagging it again moves it rather than duplicating it.

**2. Switch to it.**
```bash
pergola tag-view work
```
You should see only the tagged windows, in the tiles they kept, plus a desktop notification naming
the workspace. The reply is `tag work (1 view)`. A window opened while the tag is shown joins it.

**3. Go back to everything.**
```bash
pergola tag-view all
```
You should see every view again (`tag off — every view is shown`); `off` and `*` are accepted spellings of
the same thing.

**4. List and delete.**
```bash
pergola tag-view
```
With no name it prints one line per tag, `*` in the first column of the active one:
```text
* work         2 views
  writing      1 view
```
Then `pergola tag-remove work` replies `removed tag work (2 views now untagged)`. The windows stay
open and keep their tiles; this is the way out of a workspace a closed window left behind.

**5. Put the workspaces on number keys.** `Super+1`…`9` belongs to most desktops until you say
otherwise, so the chords are opt-in. Add this to `~/.config/pergola/config.json`:
```json
{ "tags": true }
```
and run `pergola reload` to re-grab the keys. That binds `Ctrl+Alt+1`…`9` (falling back to
`Super+1`…`9`) to `tag-view 1`…`9`, and the same numbers with `Shift` to `window-tag 1`…`9`. A
switch notifies, so the change explains itself.

## Verify

```bash
pergola tag-view
```
The `*` sits on the workspace whose windows are on screen and the view counts match what you tagged.
`pergola status` carries `tag work` on its line while a tag is active, and `pergola list` marks the
views elsewhere as `parked` — still in the tree, below the work area.

**Where this bites.** Parked windows are *moved off screen*, not unmapped, and how much stays
visible is the window manager clamping: under xfwm4 a 29 px title bar remains, under Openbox 66 px
of an 862 px client on a 900 px screen. `tag-view` on a fresh name shows an empty desktop on
purpose; the reply names the way back. With `tags` on, `pergola keys --check` proves the nine chords
were actually grabbed.
