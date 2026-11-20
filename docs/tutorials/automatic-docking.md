# Automatic docking

By default Pergola manages only what you give it: new windows float. `auto_dock` decides whether a
window that *appears* is docked into the tree as it opens, so a session of terminals tiles itself.

**Starting state.** The daemon running, `auto_dock` at its default `off`, one or two tiles docked,
and one floating window to close and reopen.

## Steps

**1. Check the value and where it comes from.**
```bash
pergola config
```
One line per known key: the value, then its source — `default`, or `config.json:<line>` when the
file sets it. Nothing is inferred from runtime.

**2. Ask for new windows to be docked.**
```bash
pergola settings auto_dock new
```
`settings` validates one key, writes it, then reloads; the reply is `auto_dock = new · config reloaded`.
A rejected value also raises a desktop notification, because a menu-launched command has no visible
stderr.

**3. Restart the daemon so it reads it.**
```bash
pergola stop
pergola daemon
```
`auto_dock` is read at daemon start, not on reload: whether a window that appears is docked depends
on the policy in force when the daemon started. `pergola stop` leaves every window exactly where it
is, so this restart is safe; a systemd user unit does the same with `systemctl --user restart pergola`.

**4. Open a new window.** Launch a terminal: you should see it dock into the tree instead of floating, joining the
last focused space, and `pergola status` shows one more panel with no matching floating window.

**5. Dock the whole session instead.**
```bash
pergola settings auto_dock all
```
Restart again. `all` is `new` plus the windows that already existed when the daemon started — the
policy that makes a fresh login tile itself. `off` turns both off. Anything outside `off`, `new`,
`all` is refused; the old `true`/`false` spellings still parse.

**6. Keep the mouse out of it, or not.**
```bash
pergola settings edge_dock false
```
That is a *different* switch: it stops a floating window dropped on a screen edge from docking (the
3 px preview goes with it) and takes effect on reload. Automatic docking is about arrival; edge
docking is about the gesture.

## Verify

```bash
pergola config
```
The `auto_dock` row reads `new` (or `all`) with `config.json:<line>` as its source, and `pergola config --check`
prints `ok    no problems` and exits 0. The live half is `pergola status`: open a window and the
panel count rises without the floating count rising with it.

**Where this bites.** `new` means windows opened *after* the daemon started, so setting it and
wondering why the existing session did not tile is the normal mistake — restart with `all` instead.
Dialogs and transients (`WM_TRANSIENT_FOR`) are never docked, whatever the policy. And `auto_dock`
does not change the target rule: new windows join the last focused space, so with nesting they can
land deeper than expected.
