# History lessons — what the first decade of tiling WMs already paid for

Every tiling window manager that came before Pergola spent its first years
re-learning the same twenty-odd X11 facts, each one as a bug report. This file
is the harvest: 700 issues filed against i3, awesomewm, bspwm, xmonad,
herbstluftwm, qtile and spectrwm between roughly 2009 and 2019, classified into
failure classes, and mapped to what Pergola does about each one.

The point is not archaeology. A bug class that cost i3 three releases and cost
awesomewm a regression is a bug class Pergola should not discover again on a
user's desktop. Where a class is already handled, the fix and the code that
holds it are named. Where a class is **not** handled, this file says so instead
of implying coverage — an honest gap is cheaper than a false reassurance.

Reproduce the harvest with `scripts/research-issues.py` (GitHub REST API, no
token needed); the raw digest and full class listing are in
`docs/research/issue-digest.md` and `docs/research/issue-classes.txt`.

## Method and its limits

* Source: GitHub REST `/repos/<owner>/<repo>/issues?state=all&per_page=100`,
  newest 100 per repo. That is a **recency slice**, not the whole history: it
  over-represents the issues people were still filing near the end of the
  window and under-represents the very early "this WM does not start" bugs.
* Classification is keyword matching over title + body, so one issue lands in
  several classes. Class counts are therefore "how often this topic appears",
  not "how many distinct bugs".
* Severity was **not** scored. A class with 160 hits is not necessarily worse
  than one with 26; it is only more discussed. The classes that matter most are
  the ones that are silent — a window that vanishes, a key that stops working,
  a layout that collapses — because those are the ones users do not report and
  simply leave over.

## Class-by-class: what history says and what Pergola does

### Keyboard layout and lock keys — 34 + 81 issues

The single most repeated family. `i3#79`, `awesome#157`, `awesome#219`,
`awesome#111`, `bspwm#16`, `spectrwm#103`, `herbstluftwm#85`, `i3#943`.

Three distinct bugs hide in it:

1. **Modifier mask matched literally.** A grab for `Mod4+Left` registers
   `Mod4Mask`, but the key event arrives with `Mod4Mask|Mod2Mask` while Num Lock
   is on, so the comparison fails and the chord silently does nothing. Users
   conclude "the WM is broken when Num Lock is on" (`bspwm#16`).
2. **Keymap changed under a live grab.** Switching to a Cyrillic/Neo/Bone
   layout renumbers keycodes; the old grabs now point at the wrong keys, so
   bindings either fire for the wrong key or not at all (`i3#79`, `awesome#157`).
3. **A held chord fires on auto-repeat.** The server repeats a held key ~11
   times a second (`awesome#230` is the CPU end of this), so "next view" held
   for a second races through eleven views and "back" walks the whole history.
   The trap is that the server marks a repeat as a `KeyRelease` and a `KeyPress`
   carrying the *same* timestamp — so a naive "clear the latch on release"
   guard is defeated by the repeat's own synthetic release (measured under
   xfwm4+Xvfb: alternating press/release pairs, each pair sharing one time).
   A chord bound to a discrete action therefore fires eleven times: `back`
   walks eleven history entries, `dock` re-docks on every repeat.

Pergola's answers:

* `pergola_x11::event_mods()` masks the event state down to
  `Shift|Control|Mod1|Mod4` before comparing, so Caps Lock and Num Lock cannot
  break a chord. This is used on the `KeyPress` path in the daemon.
* `pergola_x11::lock_mod_masks()` reads the modifier map to find the actual lock
  mask instead of assuming Mod2, and `grab_with_locks()` grabs every
  lock-combination of each binding, so the grab is valid whether or not a lock
  is engaged.
* `Event::MappingNotify` triggers `reinstall_bindings()`, which rebuilds the
  keymap, ungrabs everything and re-grabs — so a layout switch leaves working
  bindings rather than stale ones.
* `daemon::is_auto_repeat()` recognises a repeat by the server's own marker: a
  `KeyPress` whose keycode and timestamp equal the immediately preceding
  `KeyRelease`'s. A repeat press is dropped, so a held chord acts once; a
  deliberate press follows a release with a later timestamp and fires.
  `scripts/e2e.sh` holds the `back` chord for 1.2 s and asserts it moved one
  history entry, not eleven.

### Fullscreen and maximise — 70 issues

`awesome#245` (maximised Firefox not restored after F11), `i3#926`,
`awesome#203` (maximising on monitor 2 moves the window to monitor 1),
`bspwm#22`.

The failure is always the same shape: the WM treats a fullscreen or maximised
window as an ordinary tile, so it resizes a window that the application and the
user both believe is outside the tiling system. Restoring afterwards puts it
back at the wrong size or on the wrong output.

Pergola's answers:

* `WindowInfo` carries `fullscreen` (`_NET_WM_STATE_FULLSCREEN`) and
  `maximized`; `apply()` skips such windows entirely, so it never resizes a
  window whose geometry another authority owns.
* `note_configure()` ignores fullscreen and transient windows, so their
  geometry is not mistaken for an edge-dock drag — the failure mode where
  entering fullscreen "docks" the window into a tile.
* The daemon's "docking a maximised window clears the state" e2e assertion
  covers the round trip.

### Transient windows and dialogs — 91 issues

`awesome#22` (regression: dialogs not handled), `i3#787`, `awesome#71`
(Eclipse completion popup disappearing).

Dialogs are transient children of a parent window. A tiler that tiles them
destroys the interaction — the popup is placed relative to a parent that has
since moved, or it is squeezed into a tile a fifth of its intended size.

Pergola's answers:

* `WindowInfo.transient` is read from `WM_TRANSIENT_FOR`.
* `reconcile()` refuses to adopt transient windows into the tree, and `apply()`
  skips them, so a dialog stays where its toolkit put it.
* `docs/HISTORY-LESSONS.md` (this file) is the reference the code comment in
  `pergola-x11` points at when it explains why such windows are never tiled.

### Geometry and size hints — 94 issues

`awesome#200` (cannot resize LibreOffice), `awesome#197` (gaps in fair layout),
`awesome#213`, `i3#718`.

Clients declare a resize grid in `WM_NORMAL_HINTS`: a terminal wants whole
character cells, so its usable width is `base + n*inc`. A WM that computes a
tile as 797 px and hands the client 797 px gets back a window that is 794 px
wide and 3 px of the tile unpainted — the documented "11 px shortfall under
Openbox" is the same bug with a larger increment. Worse, the client clamps
*itself*, so the WM's bookkeeping and the screen disagree forever after.

Pergola's answers:

* `SizeHints` plus `snap_client_size()` / `snap_to_hints()` fit the client area
  to the increment grid **before** the frame is placed, so the tile the engine
  computed and the tile on screen agree.
* `apply()` runs `snap_to_hints()` on every target.
* `scripts/health.sh` and `scripts/e2e.sh` both derive their geometry tolerance
  from the client's own increment rather than a fixed ±2, so the test measures
  the same thing the code promises. (An increment-blind tolerance is how the
  original e2e suite reported six false failures the moment snapping landed.)

### Multi-monitor and RandR — 132 issues

`awesome#170`, `awesome#203`, `bspwm#137`, `bspwm#85`, `i3#926`.

Hotplugging an output fires a storm of events: the work area briefly reports
0×0, the tree is re-laid out against a screen that no longer exists, and the
result is either a panic or a collapsed layout. Windows are also moved to the
wrong output when a maximised window is restored.

Pergola's answers:

* `apply()` early-returns on a 0×0 work area instead of dividing by zero and
  collapsing the tree — the RandR mid-change case.
* Views are per-work-area records in the engine, so a monitor's tile set is
  independent of another's.

### Focus, raise and stacking — 170 + 85 issues

`awesome#134` (focus race), `awesome#164` (unfocusing does not work),
`awesome#189` (mouse::enter with redirected windows), `bspwm#139` (floating
window stuck beneath tiling windows), `awesome#92` (cannot minimise all windows
on a tag).

Focus is a race between the WM's intent and the application's `XSetInputFocus`,
and a stack order that the WM does not own is a stack order that will drift.
The "cannot minimise" reports are the same bug seen from the other side: a
minimised window is unmapped, and a WM that decides liveness from "is it mapped"
concludes the window is gone.

Pergola's answers:

* `reconcile()` decides "gone" from `_NET_CLIENT_LIST`, not from the mapped
  state, so a minimised window keeps its tile, title and class — it just stops
  being laid out. `list()` prints the seat as `hidden` so it does not silently
  vanish from the report.
* `reconcile()` refreshes title and class on `PropertyNotify` immediately,
  rather than waiting for the next geometry pass (`herbstluftwm#35`).

### Restart and state — 91 issues

`i3#79`-adjacent, plus the general "my layout is gone after restart" family.

The lesson is that a tiler's state must be recoverable from the layout file, not
from its own memory. Pergola's layout is a JSON document (`LayoutFile`) that the
daemon reads on start; the e2e suite restarts the daemon mid-test and asserts
that the same three tiles come back at the same geometry.

### Config and error paths — 171 issues

`i3#491` (config wizard generates an invalid config), `awesome#141` (crash on
missing theme PNG), `awesome#140` (menubar will not open when a colour is not a
colour).

Bad configuration is a crash class: a parser that trusts its input takes the
whole WM down over a typo. Pergola's answer is a typed `LayoutFile::from_json`
with an explicit `Err` path — the daemon logs `ignoring malformed layout: …`
and keeps running — plus `config`-level validation of edges and view slots that
the e2e suite exercises as its own numbered section.

### Crash and hang — 160 issues

`awesome#230` (high CPU when a key is held), `awesome#135` (keyboard
unresponsive after closing a tray program).

A WM event loop that blocks on a client, or that re-enters itself from a
property handler, hangs the whole desktop. Pergola's loop is a single
`poll`-driven select over the X connection and the control socket, with
handlers that only mutate daemon state and mark the tree dirty; the layout is
recomputed once per pass.

### Title and window-class properties — 41 + 101 issues

`herbstluftwm#35`, `awesome#205` (cannot close window with the default chord).

Title and class change after map — a browser tab switch, a terminal `cd`. A WM
that reads them once at map time keeps the stale name forever. Pergola
subscribes to `STRUCTURE_NOTIFY | PROPERTY_CHANGE` in `watch_window()` and
refreshes on `is_title_atom()`.

## Classes Pergola does not handle, and why

These are real classes in the harvest with no corresponding mechanism in the
current code. They are listed so the gap is visible rather than assumed away.

| Class | Issues | Why it is out of scope today |
|---|---|---|
| workspace / tag | 236 | Named tags own sets of views; `window-tag` moves a window onto one, `tag-view` switches, and the active tag decides what is laid out. A view on another tag is parked, not unmapped, and keeps its tile so switching back is exact. **Closed in 0.5.0-dev** (document-format `MINOR`: `doc.tags`/`doc.active_tag` behind `serde(default)`). `{"tags": true}` also puts `Super+1`…`9` on `tag-view` and `Super+Shift+n` on `window-tag`, and a switch raises a notification naming the workspace. Docking a window beside the desktop leaf retargets to the tiles, because the leaf is a floating layer with its own share and never a tile: splitting a zero-weight leaf gave the window zero width (it never appeared) and the other-axis path handed the desktop half the screen — two unit tests in `engine.rs` and the real-screen geometry check guard it. Known gaps: `arrange`/`float-all` act on every tag, the camera and gap are global, and there is no persistent on-screen workspace indicator (the overlay draws no text; see `docs/UX-NOTES.md`). |
| ipc / scripting | 227 | The control socket was request/response only, so every bar re-implemented state from `pergola tree`. **Closed in 0.4.0-dev:** `pergola subscribe` pushes committed transitions as JSON lines, with a bounded per-client queue so a stalled bar cannot stall the WM. |
| tray / panel | 132 | Pergola does not own the tray or the panel; it reads the work area the desktop's panel leaves. **Deliberately not planned** — a tiler that owns the bar is a desktop environment. |
| ewmh / icccm | 168 | `_NET_CLIENT_LIST`, `_NET_WM_STATE_*`, `_NET_FRAME_EXTENTS` and `_NET_WORKAREA` are read, `_NET_ACTIVE_WINDOW` is *set* when the daemon focuses a window, and a pager's `_NET_ACTIVE_WINDOW` *request* is read from the root's shared `SubstructureNotify` selection — a taskbar's click-to-focus works, and a request for a hidden tab view switches the tab first. **Closed in 0.4.0-dev.** |
| compositing | 57 | No compositor interaction; the daemon runs with whatever the desktop provides. **Deliberately not planned** beyond the overlay dim in item 4. |
| build / packaging | 172 | The repo-root `./install.sh` targets a local build into `~/.local/bin`; **`pergola-git` is packaged for the AUR** and installs to system paths. A prebuilt `pergola-bin` needs a release pipeline, **filed as `BACKLOG.md` item 1**. |
| lock / screensaver | 62 | Screen locking is the desktop's job; Pergola only avoids grabbing keys the locker needs. |
| performance | 26 | The tree is recomputed per event pass; there is no incremental update for very large trees. |

## The transferable rule

The classes collapse into one sentence: **the WM is not the only authority over
a window**. The application owns its size hints, the toolkit owns its dialogs,
the desktop owns its work area, the user owns their keymap, and another client
owns fullscreen. Every bug above is a WM that assumed it was the only authority
and overwrote one of those. Pergola's `apply()` and `reconcile()` are written as
"ask before you act" passes for exactly that reason.
