# UX notes — human-facing interface

The record of what a person hits when using Pergola, written while building the
keyboard/mouse release (`0.3.0-dev`). Every row comes from something observed in
the code or on a real X session during this work, not from a hypothetical user.
Severity is "how much does this cost the person who meets it": 1 cosmetic,
3 "they will give up on the feature", 5 "they cannot do the task at all".

## What works today (0.3.0-dev)

* Tile the focused window from the keyboard: `Ctrl+Alt+Shift+arrows` dock,
  `Ctrl+Alt+Shift+W` floats. Verified on xfwm4 in `scripts/e2e.sh`.
* Tile by mouse: drag a floating window to a work-area edge (docks), drag the gap
  between tiles (resize), drag a tile off its edge (floats where dropped).
* See the actions by name and click them: `Ctrl+Alt+M` / `pergola palette`, when
  `rofi`/`zenity`/`yad`/`dmenu` is installed.
* Camera, views, history, arrange, float-all and the overlay all have chords.
* `pergola-health` proves the daemon, layout invariants and work-area fit.

## Pain points

| Area | Pain point | Who hits it | Sev | Proposed fix |
|---|---|---|---|---|
| Discoverability | Nothing on screen says a gap is draggable; the only cue is the cursor once the pointer is already on a 6 px strip | New user | 3 | Highlight the band on hover even with the overlay off; a first-run hint line in the palette |
| Overlay | The dim layer needs a compositor that owns `_NET_WM_CM_S0` (measured: mean screen brightness 0.162 → 0.098 under xfwm4 with compositing on, back to 0.159 after `overlay off` — a 0.60x factor, as 40% black predicts). A bare X server, or a compositor without SHAPE, gets the 2 px lines alone, which are easy to miss; the 40% strength is a constant, not a config key | Everyone on a compositor-less session | 2 | A cue that reads without compositing, and `dim` in `config.json` |
| Discoverability | Every action is a chord that has to be learned; the palette exists but is one more chord (`Ctrl+Alt+M`) | New user | 3 | Show the palette on first start, or add a panel/applet entry it can be launched from |
| Chords | The bound chords differ per desktop (xfwm4 owns `Ctrl+Alt+arrows`, so camera keys are `Super+…`). A `bindings` map in `config.json` remaps them, `pergola bind`/`unbind` change one chord without editing JSON, and `pergola settings` (0.4.0-dev) lists every command with the chords actually bound and rebinds the one you pick. A chord the desktop already owns is still only reported in the reload output, not as a notification, and the menu does not say a chord is owned *before* you pick it | Anyone whose desktop claims chords | 1 | Notification for a conflict, and an owned-chord mark in the settings rows |
| Palette | The list is fixed: "Float this window" is offered for an already-floating window, and no row shows what it will act on | Menu users | 2 | Build rows from the focused panel's state; show the window title in the menu prompt |
| Palette | Needs an external menu program; without one the hotkey fails into the daemon log, where the user never looks | Minimal installs (no rofi) | 3 | `notify-send` fallback naming the fix; `install.sh` warns; palette row "Install rofi" |
| Ratios | `pergola ratio 0.3` rebalances a split from the keyboard, but there are no palette rows for half/third and no nudge chord; `gap: 0` leaves no drag strip | Keyboard-only users | 2 | Palette rows (half/third/two-thirds) and a nudge chord |
| Re-dock | Moving a tile to another edge takes two drags (out to float, then to the edge); a docked tile cannot be re-docked directly | Mouse users in nested layouts | 2 | If a docked window settles near a different edge, re-dock it against that edge |
| Drop target | Edge drops dock against the last focused space, so in a deep tree the window nests there instead of tiling the root — correct by design, surprising in practice | Nested layouts | 3 | Show the target rectangle during the drag (needs pointer tracking), or offer "always dock at the root" |
| Undo | No layout undo: `back`/`forward` are camera history. A wrong dock has to be reversed by hand | Everyone | 3 | Snapshot ring of trees + `Ctrl+Alt+Z`; palette "Undo layout change" |
| New windows | `auto_dock` is now a tri-state (`off`/`new`/`all`), but there is no palette toggle: changing it means editing `config.json` and reloading | Everyone who tiles | 2 | A palette toggle for `auto_dock` |
| Feedback | Success and failure of a hotkey action are printed to a terminal the user does not have | Everyone without a terminal | 2 | Optional `notify-send` toast for dock/float/errors |
| Settings | `pergola settings` (0.4.0-dev) is the menu-driven editor: every scalar key and every chord is reachable, a bad value is rejected before the file is touched, and the daemon reloads afterwards. What is still missing is help: the rows show `gap = 6`, not what a gap is or what range is allowed, and the only feedback for a rejected value is the process's stderr | Everyone who tiles | 1 | One-line help per key in the menu, and a rejected value reported as a toast |
| Zoom + drag | A drag-out while zoomed converts screen coordinates as if they were world coordinates, so the floating window can land outside the desktop leaf | Untiling while zoomed | 2 | Inverse camera transform for the drop rectangle |
| Terminal split | `split-terminal` assumes `xfce4-terminal` when `$TERMINAL` is unset, which is wrong on most desktops | Non-XFCE users | 2 | Fall back to the desktop's terminal from `XDG_CURRENT_DESKTOP`, and say which config key to set |
| Accessibility | No keyboard way to select a tile other than activating its window; the overlay draws no text and exposes no AT-SPI | Keyboard-only / low-vision | 3 | Focus-by-chord across tiles; overlay labels with the tile's size |
| Tabs | Tab groups work from the command line and default chords (`tab`, `tab-select`, `tab-next`, `tab-prev`) but have no palette row and no drag-to-tab | People expecting TrellisWM tabs; the mouse path is missing | 2 | BACKLOG #1 (commands done, UI pending) |

## Next three

1. **Settings** (rows 3, 12): one palette-driven editor for `config.json`
   (including the chord map, now that it lives there) makes the JSON file — the
   last place a person must be a developer — disappear.
2. **Ratios and context-aware palette rows** (rows 4, 6): palette rows for the
   ratios and rows that change with the focused panel make the menu sufficient
   on its own.
3. **On-screen affordances** (rows 1, 2, 7, 9): hover highlight for the gaps, a
   drop target during an edge drag, an undo chord. These are the difference between
   "works if you know it" and "you can find it".

Measured on Xfwm4. The pointer paths were not exercised under Openbox; treat that
as untested, not working.
