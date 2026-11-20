# Configuration

Pergola reads `~/.config/pergola/config.json`. Every key is optional: a missing
key falls back to the compiled default. The file is plain JSON — no comments, no
trailing commas. A malformed file is ignored as a whole; `pergola reload` refuses
it and the running settings stay in force. See [SETUP.md](SETUP.md) for
installation and [KEYBINDINGS.md](KEYBINDINGS.md) for the default chords.

## A complete file

Every key, at its default. Valid JSON as printed; copy it and edit.

```json
{
  "gap": 6.0,
  "minimum_share": 0.05,
  "auto_dock": "off",
  "overlay": false,
  "dim": 0.4,
  "desktop_share": 0.0,
  "terminal": "xfce4-terminal",
  "hotkeys": true,
  "edge_dock": true,
  "tags": false,
  "notify": true,
  "bindings": {}
}
```

`terminal` has no single compiled default: it is `$TERMINAL`, else the first
installed of `xfce4-terminal`, `x-terminal-emulator`, `gnome-terminal`, `konsole`,
`mate-terminal`, `lxterminal`, `alacritty`, `kitty`, `urxvt`, `xterm`. The example
names the first candidate; set the one you actually use.

## The four layers

1. **Compiled defaults** — `Config::default` in `crates/pergola/src/config.rs`.
   Used when the file is absent or a key is missing.
2. **`config.json`** — parsed and validated. `pergola config` prints each key's
   value and its source: `default` or `config.json:<line>`.
3. **Live daemon values** — the daemon's in-memory config. `pergola reload`
   replaces it from layer 2; `pergola gap 12` and `pergola overlay on` change it
   for the session only. A restart discards layer 3.
4. **`layout.json`** — `~/.local/state/pergola/layout.json`, the document and
   camera, written by every layout change and by `pergola save`. It holds no
   setting.

A change that seems not to stick is usually in layer 3: a session-only command
was used instead of a persisted one. Layer 2 is what survives a restart.

## Per-setting reference

| Setting | Type | Default | Accepted | Applies | Persist from the CLI |
|---|---|---|---|---|---|
| `gap` | number | `6.0` | 0 to 400 px | reload | `pergola settings gap 12` |
| `minimum_share` | number | `0.05` | >0 and ≤0.5 | reload | `pergola settings minimum_share 0.1` |
| `auto_dock` | string | `"off"` | `off`, `new`, `all` | restart/start for new windows | `pergola settings auto_dock new` |
| `overlay` | boolean | `false` | `true`/`false` | restart | `pergola settings overlay true` |
| `dim` | number | `0.4` | 0 to 0.9; needs a compositor | restart | `pergola settings dim 0.6` |
| `desktop_share` | number | `0.0` | ≥0 and <1 | reload | `pergola settings desktop_share 0.15` |
| `terminal` | string | `$TERMINAL`, else installed | on PATH or an executable path | reload | `pergola settings terminal kitty` |
| `hotkeys` | boolean | `true` | `true`/`false` | restart | `pergola settings hotkeys false` |
| `edge_dock` | boolean | `true` | `true`/`false` | reload | `pergola settings edge_dock false` |
| `tags` | boolean | `false` | `true`/`false` | restart or reload (re-grab) | `pergola settings tags true` |
| `notify` | boolean | `true` | `true`/`false` | reload | `pergola settings notify false` |
| `bindings` | object | `{}` | command → `[chord,…]`; `[]` unbinds | reload | `pergola bind <command> <chord>` |

### What each one changes

`gap` is the pixels between tiles and the width of the overlay divider. It is
also the engine's constructor argument, so a change re-tiles the screen. At `0`
there is no grab strip; use `pergola ratio` to resize.

`minimum_share` is the smallest fraction a tile may be squeezed to. It bounds
both a gap drag and `pergola ratio`: `ratio 0.02` under a 0.05 minimum applies
0.05. It must be greater than 0 and at most 0.5, or a split can never be closed.

`auto_dock` decides whether a new window is docked. `off` leaves it floating;
`new` docks windows that appear after the daemon started; `all` also adopts the
windows already on screen. The old `true`/`false` still parses (`true` = `all`).
The `new`/`all` split is measured from daemon start, so a restart resets it.

`overlay` draws split lines in the gaps and a frame around the viewport, plus a
dim layer when a compositor can render it. `dim` is that layer's strength, 0 to
0.9. **`dim` needs a compositor** (`_NET_WM_CM_S0`, e.g. xfwm4's compositor); with
none the layer is unavailable and the split lines are drawn 4 px wide instead of
2 so the boundary still reads. At `dim` 0 the layer is off. Both keys apply when
the overlay is rebuilt, so a restart is the reliable way to change them.

`desktop_share` is the share of the screen `arrange` gives the desktop leaf; `0`
means no strip. It shapes `arrange`, not a single dock.

`terminal` is the program `split-terminal` launches. It must exist on PATH or be
an executable path, or `pergola config --check` fails. `xfce4-terminal` is started
with `--disable-server` so each split is its own process.

`hotkeys` grabs the default chords at startup; `edge_dock` tiles a window dropped
on a screen edge; `notify` shows a desktop notification when a hotkey or palette
action fails (best effort — without a notifier the failure still reaches the
journal). `hotkeys` is a grab at startup, so restart to change it; the other two
reload.

`tags` binds `Super+1`…`9` (`Ctrl+Alt+1`…`9` first) to the nine named workspaces
and `Super+Shift+n` to move the focused window onto one. It **interacts with
`bindings`**: tag chords are added to the plan only while `tags` is on, and a
`bindings` entry for the same command replaces its candidates. A chord already
owned by another program is reported by `pergola keys --check`, not silently
lost.

`bindings` maps a command to the chords to try, in order. Naming a command
replaces its default candidates; an empty list unbinds it; unmentioned commands
keep the defaults. Any command the daemon understands can be bound, not only the
ones in the default plan.

## A minimal file

```json
{
  "gap": 10.0
}
```

## Example profiles

Illustrations, not built-in presets — nothing selects them. Each block is valid
JSON on its own; merge the keys you want.

### Conservative

```json
{
  "gap": 8.0,
  "auto_dock": "off",
  "overlay": false,
  "hotkeys": true,
  "edge_dock": true,
  "notify": true
}
```

### Keyboard-focused

```json
{
  "gap": 6.0,
  "tags": true,
  "hotkeys": true,
  "bindings": {
    "zoom-out": ["Super+Page_Up"],
    "back": ["Super+b", "Alt+b"],
    "view 3": ["Super+3"]
  }
}
```

### Mouse-focused

```json
{
  "gap": 12.0,
  "auto_dock": "off",
  "edge_dock": true,
  "minimum_share": 0.08,
  "overlay": false
}
```

### Automatic docking

```json
{
  "gap": 6.0,
  "auto_dock": "all",
  "overlay": true,
  "dim": 0.5,
  "desktop_share": 0.0
}
```

### Workspace-oriented

```json
{
  "gap": 6.0,
  "tags": true,
  "desktop_share": 0.12,
  "auto_dock": "new",
  "notify": true
}
```

## A worked change

```sh
pergola settings gap 12   # validate, write config.json, reload
pergola config --check    # exit 0 and "ok" when the file is valid
pergola reload            # re-read the file (settings already did this)
pergola config            # each key, its value and its source
```

`pergola config --check` reads the file itself and needs no daemon, so it is the
first command after a hand edit. It exits non-zero on an unknown key, an
out-of-range `gap`/`minimum_share`/`dim`/`desktop_share`, a nonexistent
`terminal`, or a chord that does not parse, and names the line. `pergola reload`
runs the same validation and refuses an invalid file. `pergola gap 12` is
session-only; `pergola settings gap 12` is what persists.

## The no-JSON routes

`pergola settings` opens a menu of every setting and chord; `pergola settings
--list` prints the same rows on stdout; `pergola settings <key> <value>` is the
scriptable form. All three validate the whole candidate file, write it atomically
and reload. A rejection is also sent as a desktop notification and the file is
not written.

`pergola bind <command> [chord…]` and `pergola unbind <command>` edit only the
`bindings` map and preserve every other key; `pergola bind <command> --default`
drops the override. Both routes refuse to rewrite a malformed `config.json`, and
with no daemon running the change is saved and applies at the next start.
