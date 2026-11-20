# Alongside a taskbar

Pergola tiles inside the *work area*: the screen minus whatever the desktop reserves for panels,
docks and struts. It never puts a tile under your taskbar, and it never moves the taskbar.

**Starting state.** A desktop with a panel (xfwm4 and Openbox are the measured ones), the daemon
running, and `pergola status` printing the work area as `<width>x<height>`. If that is smaller than
your screen resolution, the difference is what the panel reserved.

## Steps

**1. Confirm what the daemon sees.**
```bash
pergola doctor
```
It names the display, the EWMH client list and the running window manager, then the version. The
window manager matters: other EWMH X11 managers are expected but unmeasured.
```bash
pergola status
```
The second field is the work area, not the screen: a 1600x900 screen with a 26 px bottom bar reports
`1600x874`.

**2. Dock with the panel in place.**
```bash
pergola dock left
pergola dock right
```
You should see both tiles stop at the work area's edges: the left one starts below a top panel,
the pair ends above a bottom bar.

**3. Tile everything at once.**
```bash
pergola arrange
```
`arranged N panels (depth N)`: `arrange` builds one balanced tree from every docked *and* floating
window, inside the same work area.

**4. Keep a strip for the desktop layer.**
```bash
pergola settings desktop_share 0.15
pergola arrange
```
`desktop_share` is the fraction of the work area `arrange` leaves as a desktop strip — where
floating windows live. `0` gives it none, the default; the value must be at least 0 and below 1, and
`settings` writes and reloads in one step.

**5. See the split lines.**
```bash
pergola overlay on
```
Split boundaries are drawn as 2–3 px bars inside the gaps, so a tile edge reads against a dark
terminal, plus a viewport frame when the camera is framed on something. With a compositor the tiles
are also dimmed (`dim`, default 0.4); without one only the bars are drawn.

## Verify

```bash
pergola rects
```
Every rectangle lies inside the work area: the minimum `x`/`y` across the rows equals the panel's
inset (not 0 when a bar is on the left or top), and `x + w` of a full-width row equals the work area
width from `pergola status`. `pergola config` confirms `desktop_share` and `overlay` with their
sources.

**Where this bites.** Panels belong to the window manager, not Pergola: it reads the work area and
tiles inside it, so a panel that changes size moves the tiles only as far as the WM's next work-area
update. Off-screen placement is clamped per WM, too — xfwm4 keeps a 29 px title bar of a parked
window visible, Openbox shows 66 px of an 862 px client on a 900 px screen. A Wayland session is out
of scope: Pergola is an X11 client, and XWayland clients are moved at the compositor's discretion.
