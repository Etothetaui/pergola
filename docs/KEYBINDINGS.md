# Keyboard bindings

Pergola binds chords on the root window of the X server. The daemon tries a
`Ctrl+Alt` chord first and falls back to the `Super` chord for that command when
another program already owns the first one (xfwm4 owns `Ctrl+Alt+←/→` for
workspace switching, for example). **A table in this file is the plan, not a
promise: `pergola keys --check` prints the chords actually acquired in the
running session.** See [Finding the live bindings](#finding-the-live-bindings).

The whole keyboard surface is small on purpose — the pointer and the camera do
the rest. Every command below can also be run from a terminal
(`pergola <command>`) and most appear in the action palette.

## The default plan

`Primary` is tried first, `Fallback` second. Both are listed because which one
fires depends on your desktop; only one of them is bound per command.

### Camera

| Action | Command | Primary | Fallback |
|---|---|---|---|
| Show the whole tree | `everything` | `Ctrl+Alt+Space` | `Super+Space` |
| Zoom out one level | `zoom-out` | `Ctrl+Alt+Up` | `Super+Up` |
| Zoom in one level | `zoom-in` | `Ctrl+Alt+Down` | `Super+Down` |
| Back through camera history | `back` | `Ctrl+Alt+[` | `Super+[` |
| Forward through camera history | `forward` | `Ctrl+Alt+]` | `Super+]` |
| Frame the focused window | `frame-window` | `Ctrl+Alt+Return` | `Super+Return` |
| Show or hide the desktop strip | `toggle-desktop` | `Ctrl+Alt+d` | `Super+d` |

`frame-desktop` (frame the floating-window layer) has no default chord; bind it
with `pergola bind`.

### Moving focus

| Action | Command | Primary | Fallback |
|---|---|---|---|
| Focus the neighbour to the left | `neighbour left` | `Ctrl+Alt+Left` | `Super+Left` |
| Focus the neighbour to the right | `neighbour right` | `Ctrl+Alt+Right` | `Super+Right` |

`neighbour` also takes `up` and `down`; they have no default chord. Up/down move
inside a column, left/right between columns of the split that contains the
focused window.

### Docking, floating and ratio

| Action | Command | Primary | Fallback |
|---|---|---|---|
| Dock the floating window left | `dock left` | `Ctrl+Alt+Shift+Left` | `Super+Shift+Left` |
| Dock it right | `dock right` | `Ctrl+Alt+Shift+Right` | `Super+Shift+Right` |
| Dock it above | `dock up` | `Ctrl+Alt+Shift+Up` | `Super+Shift+Up` |
| Dock it below | `dock down` | `Ctrl+Alt+Shift+Down` | `Super+Shift+Down` |
| Float the focused window | `float` | `Ctrl+Alt+Shift+w` | `Super+Shift+w` |
| Shrink the focused panel's split | `ratio -0.05` | `Ctrl+Alt+comma` | `Super+comma` |
| Grow it | `ratio +0.05` | `Ctrl+Alt+period` | `Super+period` |

The ratio chords move the boundary next to the focused panel by 5% of the split,
so the same chord works whatever the split's current size is.

### Tabs, workspaces, layout

| Action | Command | Primary | Fallback |
|---|---|---|---|
| Next tab in the focused group | `tab-next` | `Ctrl+Alt+Page_Down` | `Super+Page_Down` |
| Previous tab | `tab-prev` | `Ctrl+Alt+Page_Up` | `Super+Page_Up` |
| Undo the last layout change | `undo` | `Ctrl+Alt+z` | `Super+z` |
| Arrange all windows | `arrange` | `Ctrl+Alt+a` | `Super+a` |
| Float every docked window | `float-all` | `Ctrl+Alt+f` | `Super+f` |
| Toggle the tree overlay | `overlay` | `Ctrl+Alt+o` | `Super+o` |
| Split with a new terminal | `split-terminal` | `Ctrl+Alt+n` | `Super+n` |
| Open the action palette | `palette` | `Ctrl+Alt+m` | `Super+m` |

### Named workspaces (tags) — opt in

`"tags": true` in `config.json` adds these; they are not bound by default,
because `Super+1`…`9` belongs to most desktops until the user says otherwise.

| Action | Command | Primary | Fallback |
|---|---|---|---|
| Switch to workspace *n* | `tag-view <n>` | `Ctrl+Alt+<n>` | `Super+<n>` |
| Move the focused window there | `window-tag <n>` | `Ctrl+Alt+Shift+<n>` | `Super+Shift+<n>` |

for *n* = 1…9.

## A keyboard picture

The keys that matter are reachable from the left hand while the right hand stays
on the pointer. `⌃` = Ctrl, `⌥` = Alt, `◆` = Super.

```text
       ⌃⌥            ⌃⌥⇧
   ┌─────────────────────────────┐
   │ [    ]    ↑    Return   d   │   [ back   ] forward
   │            │                │   ↑ zoom out   ↓ zoom in
   │ ←    ↓    →    PgUp  PgDn   │   ←/→ focus    ⇧←/→ dock
   │             ,    .          │   ,/. shrink/grow the split
   │  z   n   a   f   o   m   w  │   undo / terminal / arrange / float-all
   └─────────────────────────────┘       overlay / palette / ⇧w float
```

Arrows are `neighbour`; with Shift they are `dock`. Return frames, Space shows
everything, `[` and `]` walk camera history.

## Spelling a chord

A chord is modifier names and exactly one key, joined by `+`, case-insensitive:

```text
Ctrl+Alt+Left      Super+Shift+3      ctrl+alt+comma
```

Modifier names and accepted aliases:

| Modifier | Names |
|---|---|
| Ctrl | `Ctrl`, `Control` |
| Alt | `Alt`, `Mod1`, `Meta` |
| Shift | `Shift` |
| Super | `Super`, `Mod4`, `Win` |

Key names that are not a single character:

`Space`, `Return` (or `Enter`), `Tab`, `Escape` (or `Esc`), `Home`, `End`,
`Left`, `Right`, `Up`, `Down`, `Page_Up` (or `PageUp`, `Prior`),
`Page_Down` (or `PageDown`, `Next`), and the punctuation names `comma`,
`period`, `slash`, `semicolon`, `apostrophe`, `minus`, `equal`, `backslash`,
`grave`. The brackets can be written literally: `Ctrl+Alt+[`.

Everything else must be a single ASCII letter or digit. The `+` is the
separator, so the plus key cannot be spelled `plus`; use `Super+Shift+equal`.

Caps Lock and Num Lock do not break a binding: the daemon grabs the chord with
the lock modifiers as well, reading which modifier bit carries Num Lock from the
server's modifier map instead of assuming Mod2.

Bindings follow **keysyms, not printed keycaps**. A name is translated to an X
keysym, then the daemon looks for a keycode that produces that keysym anywhere in
its levels. On a layout where the symbol moved, the chord moves with the symbol;
on a layout where a symbol is absent, the chord cannot be bound and is reported
as `NONE`.

## Finding the live bindings

```sh
pergola keys            # the plan: primary chord per command
pergola keys --check    # what this session actually grabbed
```

`--check` asks the running daemon first and prints its live grabs. That output
is authoritative: if xfwm4 took `Ctrl+Alt+Up`, the daemon bound `Super+Up` and
`--check` says so, even though the table above lists both. When no daemon is
running, `--check` probes the server by grabbing each chord and releasing it;
every chord then reads `free` unless another client owns it.

To see what the X server does with a key, use `xev` (start it, press the chord,
read the `state` and `keysym` fields) and `xmodmap -pm` (which modifier bit is
which). `xmodmap -pke | grep -w Left` shows the keycode behind a name.

## Remapping

Change one command without editing JSON; `pergola bind` validates the chord and
the command, writes `config.json`, and reloads the daemon:

```sh
pergola bind "gap 12" Super+9     # a command outside the default plan
pergola bind "zoom-in" Super+Page_Down
pergola bind "zoom-in"            # show the override and where it came from
pergola bind "zoom-in" --default  # drop the override
pergola unbind "zoom-in"          # remove the chord entirely
```

For several changes, put them in the `bindings` map. A command named there
replaces its candidates entirely; an empty list unbinds it; a command not in the
default plan is appended:

```json
{
  "bindings": {
    "zoom-in": ["Super+Page_Down"],
    "back": ["Super+b", "Alt+b"],
    "frame-desktop": ["Super+e"],
    "overlay": []
  }
}
```

Restore a command by deleting its entry (or `pergola bind <command> --default`);
restore everything by replacing `bindings` with `{}`. `"hotkeys": false` stops
the daemon grabbing any of them without forgetting the map.

## When a shortcut does nothing

1. `pergola keys --check` — is the chord listed as bound, and to the command you
   expect? `NONE` means every candidate was taken.
2. Try the other candidate (`Super+…` or `Ctrl+Alt+…`). The daemon falls back per
   command, so one command can be on `Super` while its neighbour is on `Ctrl+Alt`.
3. If the desktop acted instead — a workspace switch, a maximised window — the
   host window manager owns that chord and Pergola never saw it. Bind a
   different chord: `pergola bind "<command>" Super+<key>`.
4. `pergola keys --probe Super+Page_Down` grabs one chord in a throwaway client
   and prints `free` or `TAKEN`; it is the direct test when no daemon is running.
5. `journalctl --user -u pergola -n 50` shows grab failures and rejected chords
   at startup and after `pergola reload`.
