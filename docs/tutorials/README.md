# Tutorials

Task-oriented guides: start from the state in the first paragraph, run the numbered commands or
press the numbered chords, and end with a command whose output proves the result. Every step names
what you should see on screen.

They assume `pergola` is on your `PATH` and the daemon is running (`pergola status` prints one
line). For installation, the reference tables and the glossary, read [Your first five
minutes](../FIRST-RUN.md) and the [project README](../../README.md) first.

## Building and navigating a layout

| Guide | Outcome |
|---|---|
| [Build a three-window layout](build-a-three-window-layout.md) | three tiles, the third nested inside the second — by command or by dragging to edges |
| [Navigate a deep layout](navigate-a-deep-layout.md) | frame, zoom out, zoom in and walk back through the camera instead of hunting for windows |
| [Resize with the keyboard](resize-with-the-keyboard.md) | move one boundary by 5 %, set a share outright, and change the gap |
| [Resize with the mouse](resize-with-the-mouse.md) | drag the gap strip between two tiles, with the minimum-share floor in place |

## Grouping windows

| Guide | Outcome |
|---|---|
| [Tabs](tabs.md) | one panel holding several windows, switched by chord or command |
| [Numbered workspaces](numbered-workspaces.md) | named sets of windows on `Super+1`…`9`, everything else parked |
| [Saved views](saved-views.md) | nine camera positions you can return to with one command |

## Fitting into your desktop

| Guide | Outcome |
|---|---|
| [Remap shortcuts](remap-shortcuts.md) | one command per chord, written and reloaded, verified with `pergola keys --check` |
| [Automatic docking](automatic-docking.md) | new windows (or the whole session) tiled as they open |
| [Alongside a taskbar](alongside-a-taskbar.md) | tiles inside the work area, a desktop strip allowed for, split lines visible |

## When it goes wrong

| Guide | Outcome |
|---|---|
| [Recover from a mistake](recover-from-a-mistake.md) | undo ten deep, reset the camera, unhide a workspace, float everything, survive a bad config |

## The three things that look alike

They are distinct throughout these guides, and mixing them up is the most common confusion:

* **view** — a saved camera position, slots `1`–`9`
([saved views](saved-views.md)). It changes what fills the screen, not which windows exist.
* **tag** — a named workspace, a set of views shown together
([numbered workspaces](numbered-workspaces.md)). It changes which windows are on screen.
* **tab** — a second window inside the same panel ([tabs](tabs.md)). It changes
which window that one tile shows.

The full chord plan is [docs/KEYBINDINGS.md](../KEYBINDINGS.md); configuration, install and
troubleshooting are [docs/SETUP.md](../SETUP.md).
