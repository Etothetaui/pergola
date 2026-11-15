# Research note: TrellisWM and its layout model

*Written 2026-11-14 while building Pergola. Everything below is either public
documentation or a direct, read-only observation of public artefacts. No
TrellisWM code was read, decompiled, or copied.*

## 1. What TrellisWM is

[TrellisWM](https://trelliswm.app) is a commercial macOS window manager
(observed version 0.10.0, build 560; Apple silicon, macOS 14+). Its pitch:

> "TrellisWM nests your real Mac windows inside each other, as deep as you like.
> Zoom out to see everything. Zoom into whatever you need."

It is **not** a BSP tiler like AeroSpace or yabai. It is a *fractal / camera*
model:

* real macOS windows are moved via the Accessibility API;
* the arrangement is a recursive split tree where a split may contain another
  split, so "spaces inside spaces" needs no extra mechanism;
* navigation is **camera-like** — step in / step out, back / forward, a framed
  node, and nine saved views — not workspace switching;
* the macOS *Desktop* is a leaf in the tree (the floating-window layer), so
  docking always leaves it a share of the screen.

Public guide facts (captured from `trelliswm.app/docs/`):

| Fact | Value |
|---|---|
| URL scheme commands | 15 (`everything`, `zoom-out`, `zoom-in`, `back`, `forward`, `neighbour?dir=`, `frame-window`, `frame-desktop`, `toggle-desktop`, `view?n=`, `save-view?n=`, `split-finder`, `arrange`, `float-all`, `settings?pane=`) |
| Keyboard shortcuts | 12 rows (`⌃⌥Space`, `⌃⌥↑/↓`, `⌃⌥←/→`, `⌃⌥⇧↑/↓`, `⌃⌥[`/`]`, `⌃⌥Return`, `⌃⌥D`, `⌃⌥N`, `⌃⌥⇧1–9`, `⌃⌥1–9`, plus `⌘⇧`-drag and `⌥`-pinch) |
| Settings panes | 8 (`general`, `windows`, `tabs`, `appearance`, `gestures`, `apps`, `shortcuts`, `license`) |
| Permissions | Accessibility (moves windows), optional Screen Recording (still pictures) |
| Plans | 14-day trial, free (≤5 docked windows per desktop), Pro (unlimited, tabs, saved views, URL commands) |

Interaction model worth noting: dragging a title bar to a screen or window edge
docks it and "neighbours make room"; dragging the gap resizes; dropping a window
on the middle of another creates a tab group; double-clicking a title bar frames
that window (again to go back); zoom is shown over a **static screenshot layer**
(ScreenCaptureKit), not a live screen recording.

## 2. Where the model is written down publicly

The layout grammar and its invariants are published in a **clean-room study**,
[`archsueh/trelliswm-reverse`](https://github.com/archsueh/trelliswm-reverse)
(unofficial, unaffiliated, no license file — "all rights reserved"). Its
`capture/layout-schema.json` records the document grammar derived from the app's
own persisted `layout-*.json` files:

```text
Node    := Desktop | Panel | Split
Desktop := { kind:"desktop", id:"desktop" }
Panel   := { kind:"panel", id, views:[SeatID], selected, tabbed? }
Split   := { kind:"split", id, axis:"x"|"y", children:[Node], weights:[Double] }
```

with seven invariants: exactly one `desktop` leaf; `children.count ==
weights.count`; `sum(weights) == 1`; splits nest recursively; a panel with more
than one view is a tab group whose `selected` points at the visible tab;
`framed` stores the camera's focus node; per-window bookkeeping is keyed by a
stable seat id.

That study also publishes a JavaScript implementation of the model
(`sim/index.html`) and a Swift one (`app/Sources/TrellisCore/`) — the only public
"web layout engine" for TrellisWM's model, written independently of the product.

## 3. Is there an official web layout engine on GitHub?

**No.** Searches performed 2026-11-14:

* GitHub repository search `TrellisWM` → two results, both third-party:
  `archsueh/trelliswm-reverse` (the clean-room study above) and
  `Sheehan-Personal/trellis-helper` (a helper that re-applies saved views).
* The `trelliswm.app` site itself links no repository: the page's bundle
  (`assets/page-*.js`, `story-*.js`, `main-*.js`) is a closed Vite build with no
  import of an external layout library, and `/docs/`, `/how-it-works/` and
  `/download/` contain no GitHub/GitLab links.
* No user or organisation `trelliswm` exists on GitHub.

So the site's animation engine is closed-source, and the nearest public artefact
is the clean-room JavaScript engine in `trelliswm-reverse/sim/index.html`. That
is what this project treats as the reference *description*, not as source:
Pergola was written from the grammar and invariants above, and its semantics
(docking rules, gap resize, camera targets) follow the documented model.

## 4. What Pergola changed to be a Linux window manager

| TrellisWM (macOS) | Pergola (X11) |
|---|---|
| Moves other apps' windows via Accessibility | Moves other apps' windows via EWMH `_NET_MOVERESIZE_WINDOW` (no WM takeover) |
| Zoom shown over a static screenshot layer | Zoom is a real geometry transform: the framed node is mapped onto the screen, everything else moves with it |
| Desktop leaf is a floating layer drawn by the app | Desktop leaf is a rectangle; floating windows are mapped into it (a space, not the screen) |
| Spaces are per macOS display | One tree per X display |
| Tabs via drag-to-tab, `split-finder` spawns Finder | Tabs modelled in the document; `split-terminal` spawns `$TERMINAL` |
| Pinch/zoom gestures, `⌘⇧`-drag rectangle | CLI, hotkeys, URL commands (`pergola://…`) |
| Saved views per display, 9 slots | Same, persisted in `layout.json` |
| macOS-only | Any EWMH-compliant X11 desktop |

## 5. Sources

1. <https://trelliswm.app/> — product page (fetched 2026-11-14)
2. <https://trelliswm.app/docs/> — public guide: commands, shortcuts, panes, permissions, plans
3. <https://trelliswm.app/how-it-works/> — interaction model
4. <https://github.com/archsueh/trelliswm-reverse> — clean-room study: layout grammar, invariants, evidence ledger
5. GitHub REST API search for `TrellisWM`, `trellis layout engine`, `fractal window manager` (2026-11-14)

## 6. Licence and trademark

"TrellisWM" and related marks belong to their owner; they are used here only to
identify the software whose *model* is being discussed (nominative use). Pergola
contains no code, assets or text from TrellisWM or from the clean-room study.
The study repository grants no reuse rights, so none were taken: only the
published grammar and invariants — facts about a file format — informed this
implementation.
