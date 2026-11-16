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
| Discoverability | Every action is a chord that has to be learned; the palette exists but is one more chord (`Ctrl+Alt+M`) | New user | 3 | Show the palette on first start, or add a panel/applet entry it can be launched from |
| Chords | The bound chords differ per desktop (xfwm4 owns `Ctrl+Alt+arrows`, so camera keys are `Super+…`). A `bindings` map in `config.json` now remaps them (0.4.0-dev), but there is still no UI: seeing or changing a chord means editing JSON and running `pergola keys --check` in a shell | Anyone whose desktop claims chords | 2 | Chord pane in the palette, and conflicts reported as a notification |
| Palette | The list is fixed: "Float this window" is offered for an already-floating window, and no row shows what it will act on | Menu users | 2 | Build rows from the focused panel's state; show the window title in the menu prompt |
| Palette | Needs an external menu program; without one the hotkey fails into the daemon log, where the user never looks | Minimal installs (no rofi) | 3 | `notify-send` fallback naming the fix; `install.sh` warns; palette row "Install rofi" |
| Ratios | Only a gap drag changes split weights; there is no `pergola ratio`, and `gap: 0` means no drag target at all | Keyboard-only users | 3 | `pergola ratio 0.3` plus palette rows (half/third/two-thirds) and a nudge chord |
| Re-dock | Moving a tile to another edge takes two drags (out to float, then to the edge); a docked tile cannot be re-docked directly | Mouse users in nested layouts | 2 | If a docked window settles near a different edge, re-dock it against that edge |
| Drop target | Edge drops dock against the last focused space, so in a deep tree the window nests there instead of tiling the root — correct by design, surprising in practice | Nested layouts | 3 | Show the target rectangle during the drag (needs pointer tracking), or offer "always dock at the root" |
| Undo | No layout undo: `back`/`forward` are camera history. A wrong dock has to be reversed by hand | Everyone | 3 | Snapshot ring of trees + `Ctrl+Alt+Z`; palette "Undo layout change" |
| New windows | `auto_dock` is still a bool, off by default: a new terminal floats into the middle of a tiled layout | Everyone who tiles | 3 | Tri-state `auto_dock` (off/new/all) with a palette toggle (already BACKLOG #2) |
| Feedback | Success and failure of a hotkey action are printed to a terminal the user does not have | Everyone without a terminal | 2 | Optional `notify-send` toast for dock/float/errors |
| Settings | No settings UI: every change is edit `config.json` + `pergola reload`. `Command::Settings` exists in core but nothing renders it | Everyone | 3 | `pergola settings` — palette-driven editor that writes `config.json` |
| Zoom + drag | A drag-out while zoomed converts screen coordinates as if they were world coordinates, so the floating window can land outside the desktop leaf | Untiling while zoomed | 2 | Inverse camera transform for the drop rectangle |
| Terminal split | `split-terminal` assumes `xfce4-terminal` when `$TERMINAL` is unset, which is wrong on most desktops | Non-XFCE users | 2 | Fall back to the desktop's terminal from `XDG_CURRENT_DESKTOP`, and say which config key to set |
| Accessibility | No keyboard way to select a tile other than activating its window; the overlay draws no text and exposes no AT-SPI | Keyboard-only / low-vision | 3 | Focus-by-chord across tiles; overlay labels with the tile's size |
| Tabs | Tab groups are modelled (`views.len() > 1`) but have no UI, chord or palette row | People expecting TrellisWM tabs | 2 | BACKLOG #1 |

## Next three

1. **Settings** (rows 3, 12): one palette-driven editor for `config.json`
   (including the chord map, now that it lives there) makes the JSON file — the
   last place a person must be a developer — disappear.
2. **Ratios and context-aware palette rows** (rows 4, 6): `pergola ratio` and rows
   that change with the focused panel make the menu sufficient on its own.
3. **On-screen affordances** (rows 1, 2, 7, 9): hover highlight for the gaps, a
   drop target during an edge drag, an undo chord. These are the difference between
   "works if you know it" and "you can find it".

Measured on Xfwm4. The pointer paths were not exercised under Openbox; treat that
as untested, not working.
