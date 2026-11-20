# Remap shortcuts

`pergola bind` validates a chord and a command, writes `config.json` and reloads the daemon — no JSON by hand, no restart.

**Starting state.** The daemon running with the default plan ([docs/KEYBINDINGS.md](../KEYBINDINGS.md) is that plan), and `pergola keys --check` printing one line per command: `bound <chord> <command>`, `NONE <command>` when every candidate was taken, or `off … (unbound)` when you removed it on purpose.

## Steps

**1. See what is bound now.**
```bash
pergola keys --check
```
With a daemon running this is authoritative: the chord it actually acquired, not the plan. If the desktop took `Ctrl+Alt+Down`, the line shows `Super+Down`, because that is the one that fires. The trailing note says these are the daemon's live grabs.

**2. Bind one command to one chord.**
```bash
pergola bind "zoom-in" Super+Page_Down
```
Reply: `bound 'zoom-in' to Super+Page_Down in ~/.config/pergola/config.json`, then the reload. If another client owns the chord the reply says so, the old binding stays, and `pergola keys --check` agrees.

**3. Show it, then put it back.**
```bash
pergola bind "zoom-in"
pergola bind "zoom-in" --default
```
You should see the first print the override and where it came from — `from config.json` or `default`; no chord argument means "show". The second drops the override so the default candidates apply again.

**4. Remove a chord entirely.**
```bash
pergola unbind "zoom-in"
```
`pergola keys --check` then reads `off   (unbound) … zoom-in`: a decision, not a grab failure.

**5. Change several at once.** Edit the `bindings` map and reload:
```json
{
  "bindings": {
    "zoom-in": ["Super+Page_Down"],
    "frame-desktop": ["Super+e"],
    "overlay": []
  }
}
```
```bash
pergola reload
```
A command named here replaces its candidates entirely, an empty list unbinds it, and a command outside the default plan (such as `"gap 12"`) is appended. `"hotkeys": false` stops the daemon grabbing any of them without forgetting the map; that one needs a restart.

## Verify

```bash
pergola keys --check
```
The line for the command you changed must read `bound Super+Page_Down zoom-in` — the new chord, acquired in this session. With no daemon running, `pergola keys --probe Super+Page_Down` grabs that one chord in a throwaway client and prints `free ` or `TAKEN`.

**Where this bites.** A chord is modifier names plus exactly one key joined by `+` (`Ctrl+Alt+Left`, `Super+Shift+3`, `Ctrl+Alt+comma`); see [Spelling a chord](../KEYBINDINGS.md#spelling-a-chord). Bindings follow keysyms, not printed keycaps, so a chord can be `NONE` on a layout where the symbol does not exist. Grab failures land in `journalctl --user -u pergola -n 50`.
