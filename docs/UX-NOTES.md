# UX notes — human-facing interface

The record of what a person hits when using Pergola, written while building the
keyboard/mouse release (`0.3.0-dev`). Every row comes from something observed in
the code or on a real X session during this work, not from a hypothetical user.
Severity is "how much does this cost the person who meets it": 1 cosmetic,
3 "they will give up on the feature", 5 "they cannot do the task at all".

## What works today (0.5.0-dev)

* Tile the focused window from the keyboard: `Ctrl+Alt+Shift+arrows` dock,
  `Ctrl+Alt+Shift+W` floats. Verified on xfwm4 in `scripts/e2e.sh`.
* Tile by mouse: drag a floating window to a work-area edge (docks), drag the gap
  between tiles (resize), drag a tile off its edge (floats where dropped).
* See the actions by name and click them: `Ctrl+Alt+M` / `pergola palette`, when
  `rofi`/`zenity`/`yad`/`dmenu` is installed.
* Camera, views, history, arrange, float-all and the overlay all have chords.
* Undo the last layout change (`Ctrl+Alt+Z`, palette "Undo the last layout
  change", ten steps): a wrong dock, float, arrange, ratio or gap drag is one
  chord away from where it was. Verified on xfwm4 and openbox in
  `scripts/e2e.sh`.
* Read the overlay without a compositor: the split lines are drawn 4 px wide
  when there is no dim layer to carry the cue (2 px when a compositor dims the
  tiles), and the dim strength is the `dim` key — `0.4` by default, `0` for
  lines only. Verified in `scripts/e2e.sh` (0.6 is 60% opaque, `dim 0` draws no
  layer) and by unit test on the line width.
* See where a gap is before aiming at it: hovering a band draws a thin line down
  its long axis (0.4.0-dev). The hint is click-through, so it never steals the
  pointer from the band it describes.
* See where a dropped window will land: dragging within 70 px of a work-area edge
  draws a 4-bar frame around the rectangle the drop would produce. The preview
  runs the same target selection as the real dock, so the frame is the answer.
* Re-dock a tile in one gesture: drag it out of its own rectangle and settle it
  near another edge and it docks there, without the float-then-dock detour.
* Click a taskbar entry to focus its window. The click is an `_NET_ACTIVE_WINDOW`
  request to the root window; the daemon reads it and focuses the window, and if
  the window is a hidden tab view it shows that view first. Verified on xfwm4 and
  openbox in `scripts/e2e.sh`.
* Name a workspace and switch to it: `pergola window-tag work` moves the focused
  window onto a named tag, `pergola tag-view work` lays out only that tag's
  views and parks the rest below the work area, `tag-view all` brings everything
  back, and `tag-remove` deletes the name without closing a window. Each tag
  remembers the window that had the keyboard. Verified on xfwm4 and openbox in
  `scripts/e2e.sh` (disjoint tag sets, only the active tag's panels mapped, the
  parked client area off the work area, a restart, and a `tag` event).
* Switch workspace with one key: `{"tags": true}` puts `Super+1`…`9` on
  `tag-view 1`…`9` and `Super+Shift+1`…`9` on moving the focused window, pressed
  for real in the e2e (the switch lands, the new workspace starts empty, the
  move chord brings the window with you, the same chord returns, and `tags:
  false` releases exactly those chords and nothing else). A switch also names
  itself in a desktop notification, so a screen that changed wholesale says
  where it changed to.
* Change a setting from a menu that explains itself: every `pergola settings`
  row carries what the key controls, the range the validator enforces and the
  compiled default, a chord row takes a comma-separated pair, and a rejected
  value is also sent as a desktop notification (`notify-send`, or
  `PERGOLA_NOTIFY` for another program) because the menu runs detached.
  Verified on xfwm4 in `scripts/e2e.sh` with a stand-in notifier — the value is
  still refused and the file is not touched.
* Hear about a failed hotkey or palette action: a chord runs inside the daemon
  and a palette action runs detached, so their stderr goes to the journal — the
  one place a person does not look. The `notify` key (default `true`) sends the
  failure as a desktop notification (`notify-send`, or `PERGOLA_NOTIFY`), best
  effort: without a notifier it still reaches the journal. Verified on xfwm4 and
  openbox in `scripts/e2e.sh` by binding a chord to a command that fails at run
  time (`dock sideways`).
* Resize a split without a pointer or a target: `Ctrl+Alt+,` / `Ctrl+Alt+.`
  nudge the focused window's share by 5% (the bound chord is read from the
  daemon, so a desktop that owns Ctrl+Alt gets the Super fallback), and the
  palette carries half / one-third / two-thirds rows. Verified on xfwm4 in
  `scripts/e2e.sh` by measuring the focused window's own width after a real key
  press.
* The action palette is built from the focused window, not a fixed list: a
  floating window is offered "Fold this window into a tab" instead of the no-op
  "Float this window", the prompt names the window, and the full-size list is
  reachable again by asking. `palette --list` prints the same rows on stdout, so
  the state can be checked without a menu program. Verified on xfwm4.
* `pergola-health` proves the daemon, layout invariants and work-area fit.

## Pain points

| Area | Pain point | Who hits it | Sev | Proposed fix |
|---|---|---|---|---|
| Overlay | The dim layer needs a compositor that owns `_NET_WM_CM_S0` (measured: mean screen brightness 0.162 → 0.098 under xfwm4 with compositing on, back to 0.159 after `overlay off` — a 0.60x factor, as 40% black predicts). A compositor without SHAPE gets no dim layer either. The `dim` key and the 4 px no-dim lines are in place (0.5.0-dev); what is still unmeasured is whether 4 px reads well enough on a *real* bare-X session — the width is asserted by unit test, not by a person looking at it | Compositor-less sessions | 1 | Look at a 4 px line on a real bare X server and tune the width/gap if it is still easy to miss |
| Discoverability | Every action is a chord that has to be learned; the palette exists but is one more chord (`Ctrl+Alt+M`) | New user | 3 | Show the palette on first start, or add a panel/applet entry it can be launched from |
| Chords | The bound chords differ per desktop (xfwm4 owns `Ctrl+Alt+arrows`, so camera keys are `Super+…`). A `bindings` map in `config.json` remaps them, `pergola bind`/`unbind` change one chord without editing JSON, and `pergola settings` (0.4.0-dev) lists every command with the chords actually bound and rebinds the one you pick. A chord the desktop already owns is still only reported in the reload output, not as a notification, and the menu does not say a chord is owned *before* you pick it | Anyone whose desktop claims chords | 1 | Notification for a conflict, and an owned-chord mark in the settings rows |
| Palette | Needs an external menu program; without one the hotkey fails into the daemon log, where the user never looks | Minimal installs (no rofi) | 3 | `notify-send` fallback naming the fix; `install.sh` warns; palette row "Install rofi" |
| New windows | `auto_dock` is now a tri-state (`off`/`new`/`all`), but there is no palette toggle: changing it means editing `config.json` and reloading | Everyone who tiles | 2 | A palette toggle for `auto_dock` |
| Feedback | Success and failure of a hotkey action are printed to a terminal the user does not have. A *rejected setting* and a *failed hotkey/palette action* are now also desktop notifications (0.5.0-dev, `notify` key), but the toast is one line naming the command and its error — it does not say which window or offer a retry | Everyone without a terminal | 2 | A richer toast naming the window and a one-click retry; the plain message is in place |
| Zoom + drag | A drag-out while zoomed converts screen coordinates as if they were world coordinates, so the floating window can land outside the desktop leaf | Untiling while zoomed | 2 | Inverse camera transform for the drop rectangle |
| Terminal split | `split-terminal` assumes `xfce4-terminal` when `$TERMINAL` is unset, which is wrong on most desktops | Non-XFCE users | 2 | Fall back to the desktop's terminal from `XDG_CURRENT_DESKTOP`, and say which config key to set |
| Accessibility | No keyboard way to select a tile other than activating its window; the overlay draws no text and exposes no AT-SPI | Keyboard-only / low-vision | 3 | Focus-by-chord across tiles; overlay labels with the tile's size |
| Tabs | Tab groups work from the command line, from default chords (`tab`, `tab-select`, `tab-next`, `tab-prev`) but there is no palette row for a tab action, no drag-a-window-onto-a-tab gesture and no tab bar | People expecting TrellisWM tabs; the mouse path is missing | 2 | Not scheduled (drag-to-tab is not in BACKLOG.md) |
| Workspaces | Workspaces are one key away (`tags: true` binds `Super+1`…`9` and `Super+Shift+n`; a switch raises a notification naming the workspace), and the palette offers a row per workspace the document holds. What remains: a workspace has no persistent on-screen indicator — the overlay draws no text and a strip of unnamed rectangles would say less than the notification; a status bar can draw a real one from `pergola subscribe` | Anyone who expects a workspace bar | 1 | A text-drawing capability in the x11 crate, then a small tag strip in the overlay (deliberately not a stopgap of blind rectangles) |
| Workspaces | A parked window is moved below the work area, not unmapped, and the desktop clamps it so it cannot be lost: xfwm4 keeps the 29 px title bar at the bottom edge, openbox keeps 66 px of an 862 px client (measured on a 900 px screen; a larger park offset does not help — `xdotool windowmove` to y=900, 1200 or 3000 all land the client at the same clamped y). So the window is off the workspace but a sliver of it is on screen | Anyone with more than one workspace | 2 | An ICCCM iconify (`WM_CHANGE_STATE`) on park and a pager-style activation on return, tracking only the windows we hid so the user's own minimise is never undone; or unmapping, which needs the host WM's client list to stay honest |

## Next three

1. **A richer toast for a failed dock/float** (the Feedback row) — the plain
   `PERGOLA_NOTIFY` message is in place (0.5.0-dev); what remains is naming the
   window and offering a retry. The overlay cue and the `dim` key are done; what
   remains there is the look-at-it check on a real bare-X server.
2. **A workspace indicator that is not a notification** (the Workspaces row):
   the chords and palette rows are done; the remaining gap is persistent
   on-screen state, which needs text drawing in the x11 crate rather than a
   strip of unnamed rectangles.
3. **A palette that can be launched without a chord** (the Discoverability and
   Palette-needs-a-menu rows): a panel/applet entry and a `notify-send` naming
   the missing menu program, so the menu is not itself a hidden feature.

Measured on Xfwm4 and Openbox. Openbox coalesces a drag into a single
`ConfigureNotify`, so the mid-drag drop frame is asserted on xfwm4 only; the
hover hint and the one-gesture re-dock are asserted on both.
