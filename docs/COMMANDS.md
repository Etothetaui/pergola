# Commands

`pergola daemon` runs the layout daemon (one per display); every other subcommand
is a client that forwards to it over a Unix socket. A command that succeeds prints
its result and exits 0; a refused one prints the reason on stderr and exits 1.
Arguments are read left to right. A window id is decimal or `0x` hexadecimal. A
command containing spaces must be quoted when bound: `pergola bind "gap 12"
Super+9`. `--socket PATH` before the command points the client at another daemon.

## Camera

Camera commands change the viewport, not the tree. They are undoable, and the
camera's `back`/`forward` history is separate from `pergola undo`.

### `everything`
`pergola everything` — no args. Frames the whole tree and turns the overlay on.
Focus unchanged. Result: `everything → framed <id> (depth N)`. Errors: none.

### `zoom-in` / `zoom-out`
`pergola zoom-in` · `pergola zoom-out` — no args. Steps one level in or out of the
framed node. `zoom-in` turns the overlay off. Focus unchanged. Errors: none.

### `back` / `forward`
`pergola back` · `pergola forward` — no args. Browser-style history over framed
nodes. At either end the command is a no-op but still prints the framed node.
Focus unchanged. Errors: none.

### `frame-window` / `frame-desktop` / `toggle-desktop`
`pergola frame-window` — no args. Frames the focused window's panel; running it
again on the same window returns to the whole tree. Needs a focused window; with
none the daemon reports that it is not a camera command. Result: `frame-window →
framed <id> (depth N)`.

`pergola frame-desktop` — no args. Frames the desktop layer, or the whole tree when
the desktop is already framed. `pergola toggle-desktop` — no args. Hides or shows
floating windows; docked tiles are untouched. All three leave focus unchanged;
`toggle-desktop` is undoable. Errors: none.

### `neighbour <left|right|up|down>`
`pergola neighbour left` — one direction, required. Moves the camera to the
neighbouring space in the tree geometry; with no neighbour it does not move.
Missing or unknown direction: `usage: pergola neighbour <left|right|up|down>`.
Focus unchanged. Undoable.

## Focus and navigation

### `view <1-9>` / `save-view <1-9>`
`pergola view 2` · `pergola save-view 2` — one slot 1–9, required. `save-view`
stores the framed node in the slot; `view` returns to it. Missing argument prints
`usage: pergola view <1-9>` (or `save-view`); out of range: `slot must be 1-9`.
Focus unchanged. Undoable. Saved views are camera slots, not workspaces.

### `tab-select <seat|window>` / `tab-next` / `tab-prev`
`pergola tab-select 0x1a2b3c` — one seat or window id, required. Shows that view of
its tab group. A panel that is not a group answers `<panel> is not a tab group`
and changes nothing. Errors: `'<raw>' is not a seat or window id`, `window 0x… is
not managed by pergola`, `… is not in the layout`, `… is not a view of <panel>`.
`pergola tab-next` · `pergola tab-prev` — no args. Cycles the shown view of the
focused tab group (then the last-focused group, then any group on screen). No
group: `no tab group to cycle`. Empty group: `<panel> has no views`. All three
change focus to the shown window and are undoable.

## Docking and floating

### `dock <left|right|up|down> [--window ID] [--target ID]`
`pergola dock left` · `pergola dock right --window 0x1a2b3c --target p-4` — one
edge, required. `--window` defaults to the active window; `--target` defaults to
the focused panel, then the last focused panel, then the whole tree; a stale target
falls back to the root. Docks the window against that edge of the target, removing
it from its tile, tab group or floating layer. Errors: missing edge prints the
usage line; `no active window to dock`; an unmanaged window. Focus unchanged.
Undoable. Result: `docked <panel> left of <target> (depth N)`.

### `float [--window ID]`
`pergola float` · `pergola float --window 0x1a2b3c` — optional `--window`, default
the active window. Returns the window to the desktop layer at its current
rectangle, so it does not jump. Errors: `no active window to float`; an unmanaged
window. Focus unchanged. Undoable. Result: `floated <panel>`.

### `arrange`
`pergola arrange` — no args. Rebuilds the whole document — every panel in the tree
plus every floating window — into a balanced fractal tree and empties the floating
layer. Empty document: `nothing to arrange`. Focus unchanged. Undoable. Result:
`arranged N panels (depth N)`.

### `float-all`
`pergola float-all` — no args. Moves every window in the whole document to the
desktop layer and frames the desktop. Layout state changes; `pergola save`
persists it. Focus unchanged. Undoable. Result: `floated N panels`.

### `split-terminal`
`pergola split-terminal` — no args (`split-finder` is the alias). Splits the
focused window with a new terminal from the `terminal` setting. Errors: `focus a
window before splitting with a terminal`; the focused window is not managed;
`cannot launch …`; `the terminal did not open a window`; the new window was not
adopted. Focus: the new terminal takes the keyboard. Undoable.

## Ratio and resize

### `gap <pixels>`
`pergola gap 12` — one number, required. The live gap; the config validator
accepts 0–400. Rebuilds the engine and re-tiles the screen. Errors: `gap must be a
number`; missing argument prints `usage: pergola gap <pixels>`. Global to every
tag and split. Session-only: it does not write `config.json`; use `pergola
settings gap 12` to persist. At 0 there is no grab strip. Undoable.

### `ratio <0.0-1.0> | +delta | -delta`
`pergola ratio 0.3` · `pergola ratio +0.05` — one number, required. A bare number
sets the focused panel's share of its split; a leading `+` or `-` nudges the
current share. Siblings keep their relative proportions. Errors: `no focused
window to rebalance`; `ratio must be a number`; `ratio must be between 0.0 and
1.0`; `'<id>' has no split to rebalance`. The applied share is clamped by
`minimum_share` and printed with the split weights. Focus unchanged. Undoable.

### `undo`
`pergola undo` — no args. Restores the layout before the last change, up to 10
changes deep; it does not change configuration. Empty stack: `nothing to undo`.
Focus returns to the restored snapshot. Not itself undoable.

## Tabs

### `tab [--window ID] [--target PANEL]`
`pergola tab` · `pergola tab --window 0x1a2b3c --target p-4` — optional `--window`
(default active) and `--target` (default the focused panel unless that is the
window's own panel; a target may be a panel or seat id). Folds the window into the
target as a tab; a panel with a second view becomes a tab group and only its
selected view is raised. A floating window is folded in from the desktop layer.
The window joins the target's tag. Errors: `no active window to tab`; an unmanaged
window; `no panel '<target>' to tab into`. Focus: yes, the tabbed window. Undoable.
Result: `tabbed <seat> into <panel> (N views)`.

`tab-select`, `tab-next` and `tab-prev` are documented under
[Focus and navigation](#focus-and-navigation).

## Workspaces and tags

### `tag-view [name|all]`
`pergola tag-view` · `pergola tag-view work` · `pergola tag-view all` — optional
name. No argument lists the tags (`*` marks the active one). A name switches to
that workspace, creating it when new. `all`, `off` or `*` clears the active tag so
every view is shown. The keyboard position on the tag being left is remembered and
restored on return; an empty new tag prints the way back. Errors: none — any name
is valid. Focus: restored to the tag's remembered seat when visible. Undoable.

### `window-tag <name> [--window ID]`
`pergola window-tag work` · `pergola window-tag work --window 0x1a2b3c` — one name,
required; optional `--window`, default active. Moves the window into that tag
without switching to it; the tag is created when new. `all`, `off` and `*` are
refused: `'<name>' is not a tag name`. Errors: missing name prints the usage line;
`no active window to tag`; an unmanaged window. Focus unchanged; the window goes
off screen if the tag is not active. Undoable.

### `tag-remove <name>`
`pergola tag-remove work` — one name, required. Deletes the workspace; its views
become untagged and show again under `tag-view all`. The windows stay open.
Removing the active tag clears it. Errors: `no tag '<name>'`. Focus unchanged.
Undoable.

## Views

`view <1-9>` and `save-view <1-9>` are camera slots; see [Focus and
navigation](#focus-and-navigation). The workspace command is `tag-view`.

## Config and settings

### `config [--check]`
`pergola config` · `pergola config --check` — optional `--check`. Local: reads the
file and needs no daemon. Without it, prints the path and every key with its value
and source (`default` or `config.json:<line>`). With it, prints `ok` or each
`FAIL` line and exits non-zero on an unknown key, a bad range, a nonexistent
terminal or a chord that does not parse. Errors: `N problem(s) in <path>`.

### `settings`
`pergola settings` · `pergola settings --list` · `pergola settings gap 12` — no
args opens the settings menu (rofi, zenity or dmenu); `--list` prints the same rows
on stdout; `<key> <value>` sets one key. A one-shot change validates the whole
candidate file, writes it atomically and reloads. Errors: `settings <key> needs a
value`; an unknown key or invalid value is refused and the file is not written; a
daemon that rejects the reload is reported as `saved … but the running daemon
rejected reload`.

### `settings-run`
`pergola settings-run` — no args. The detached child the settings menu starts so a
hotkey does not block the daemon; shows the menu and applies the chosen row. Not
normally typed. Needs a menu program.

### `palette [--list]`
`pergola palette` · `pergola palette --list` — optional `--list`. Opens the
tile-action menu for the focused window; `--list` prints the state-aware rows as
`label<TAB>argv`. Errors: without a menu program it reports which one is missing.
`pergola palette-run` is the detached child that runs the chosen row; not normally
typed.

### `overlay [on|off|toggle]`
`pergola overlay` · `pergola overlay on` · `pergola overlay off` — optional action;
no argument toggles. Session-only: the `overlay` config key is the startup default.
Reports the state and the dim case (`dim N% 0x…`, `dim off (dim 0)`, or `dim
unavailable (no compositor or SHAPE)`). Undoable. Errors: none.

### `reload`
`pergola reload` — no args. Re-reads `config.json`, validates it, rebuilds the
engine and re-grabs the chords so a `bindings` edit takes effect without a
restart. An invalid file is refused and the running settings stay in force;
conflicts and fallbacks come back in the reply. Errors: `config reload refused;
the running settings are unchanged: …`.

### `save`
`pergola save` — no args. Writes the current document to `layout.json` atomically.
Result: `saved <path>`.

## Keys and binding

### `keys [--check] [--probe <chord>...]`
`pergola keys` · `pergola keys --check` · `pergola keys --probe Super+Up` — optional
`--check` or `--probe` plus chords. Without flags, asks the running daemon what it
actually bound (falls back to the plan). `--check` lists the plan and marks chords
the desktop owns; an unbound command shows `off`. `--probe` grabs each chord
briefly and prints `free` or `TAKEN`; an unparseable chord is an error. The live
answer is authoritative for the running session.

### `bind <command> [chord...]`
`pergola bind "gap 12" Super+9` · `pergola bind "gap 12"` · `pergola bind "gap 12"
--default` — a command (quote it if it has spaces), then chords, nothing, or
`--default`. With chords, writes the override into `bindings`, reloads and prints
any conflict. With no chords, shows the effective candidates and whether they come
from `config.json` or the default plan. With `--default`, removes the override.
Errors: `'<command>' is not a pergola command`; `cannot parse binding '<chord>'`.
With no daemon running the change is saved and applies at the next start.

### `unbind <command>`
`pergola unbind "view 3"` — one command. Writes an empty candidate list and
reloads. Errors: unknown command.

## Inspection

### `status` / `tree`
`pergola status` — no args. One line: version, screen size, panels and floating
count on the active tag, tabs, depth, framed node, gap, overlay, active tag, hint
state and the active window. `pergola tree` — no args. The whole document as JSON:
tree, floating layer, tags, tab selections and camera. Both are local to the
daemon; `status` counts the active tag, `tree` is the document.

### `list`
`pergola list` — no args. One line per window: `docked`, `floating`, `tab`,
`parked`, `hidden` or `unmanaged`, the window description, and `[tag <name>]` when
it belongs to one. Minimised windows are listed as `hidden`.

### `rects [--json]`
`pergola rects` · `pergola rects --json` — optional `--json`. Each panel's frame
rectangle after the camera transform, as `id x y w h` or a JSON array of
`{id,x,y,w,h}`. Screen pixels, so a framed window fills the work area.

### `sim [--json]`
`pergola sim` · `pergola sim --json` — optional `--json`. Local headless check: lays
the built-in fixture tree out in a 1600×900 frame and prints the rectangles. Needs
no daemon or display.

### `doctor`
`pergola doctor` — no args. Local probe: display, EWMH client list, running window
manager and version. Prints one note per line and the version last.

### `url <pergola://…>`
`pergola url pergola://zoom-in` · `pergola url pergola://neighbour?dir=left` — one
command URL. Parses it and runs its camera action through the daemon. The URL form
is `pergola://<command>` plus `?n=1..9` for `view`/`save-view`, `?dir=left|right|up|down`
for `neighbour`, and `?pane=…` for `settings`. Only camera commands have a URL
action: a non-camera name is refused with `'<name>' is not a camera command`; an
unknown name or a bad parameter is a parse error.

## Lifecycle and events

### `daemon` / `stop`
`pergola daemon` — no args. Runs the layout daemon in the foreground, one per
display; it never takes `SubstructureRedirect`, so it runs inside the desktop's
window manager. `pergola stop` — no args. Asks the daemon to exit; windows stay
where they are and a systemd user unit or `pergola daemon` restarts it.

### `subscribe`
`pergola subscribe` — no args. Streams the daemon's events as JSON lines — layout,
focus, dock, undock and window-closed records — until the daemon goes away. A
panel reads this instead of polling `pergola tree`. A refused subscription is an
error; the end of the stream is reported on stderr.

### `version` / `help`
`pergola version` · `pergola --version` prints the version and commit;
`pergola help` · `pergola --help` prints the grouped command summary, file paths
and compatibility note. Running `pergola` with no arguments prints the help.

## Scope: what each command changes

| Command | Acts on |
|---|---|
| `arrange`, `float-all` | the whole document, every tag |
| `dock`, `float`, `tab`, `split-terminal` | the window's own tag |
| `window-tag`, `tag-remove` | the named tag and its views |
| `tag-view` | the active tag (screen contents) |
| `gap`, `overlay`, `reload` | the daemon's global state |
| camera commands and `view`/`save-view` | the camera only |
| `undo` | the last snapshot, whatever it changed |
