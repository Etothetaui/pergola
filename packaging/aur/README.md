# AUR packaging

`PKGBUILD` builds Pergola from source and follows `main`. It is the same code
`install.sh` installs, moved from your home directory to system paths.

## Why `pergola-git` and not `pergola-bin`

AUR naming is a contract: `-bin` means *prebuilt binary, no compilation*. Pergola
has no GitHub release with a binary artifact yet (there is no CI workflow, and no
git tag), so a `pergola-bin` package would either compile from source — lying in
its name — or ship nothing. `pergola-git` is the honest name for what this
package does. A `pergola-bin` package becomes possible once a release pipeline
publishes an artifact; see "Cutting a `pergola-bin` package" below.

## Build and install

```sh
cd packaging/aur
makepkg -f            # build; leaves pergola-git-<version>-<arch>.pkg.tar.zst
makepkg -si           # build and install with pacman
```

The package is installed but not enabled. Start it for your user:

```sh
systemctl --user daemon-reload
systemctl --user enable --now pergola.service
pergola status
```

## What it installs, and where

| `install.sh` writes | the package installs |
| --- | --- |
| `$BINDIR/pergola` | `/usr/bin/pergola` |
| `$BINDIR/pergola-health` | `/usr/bin/pergola-health` |
| `$CONFIG_DIR/config.json` | `/usr/share/doc/pergola-git/config.json` (an example) |
| `$XDG_CONFIG_HOME/systemd/user/pergola.service` | `/usr/lib/systemd/user/pergola.service` |
| `$XDG_CONFIG_HOME/autostart/pergola.desktop` | `/usr/share/xdg/autostart/pergola.desktop` |
| — | `/usr/share/licenses/pergola-git/{LICENSE,NOTICE}` |
| — | `/usr/share/doc/pergola-git/{README,SETUP,FIRST-RUN,UX-NOTES,HISTORY-LESSONS}.md` |

Two deliberate differences from the source install:

1. **The config is an example, not a file in your home.** Pacman must not write
   into `$HOME`. Pergola's compiled defaults are already sane; copy the example
   to `~/.config/pergola/config.json` only when you want to change something.
   Changing a key binding does not need the file at all: `pergola bind "gap 12"
   Super+9` writes it for you.
2. **The unit does not hard-code `DISPLAY=:0`.** The source unit does, because
   `install.sh` substitutes the display it ran on. A distro package cannot know
   your display, so the packaged unit inherits `DISPLAY` from the graphical
   session (`After=graphical-session.target`). Override it with a drop-in if you
   need to:

   ```sh
   systemctl --user edit pergola.service
   # [Service]
   # Environment=DISPLAY=:1
   ```

## If you also used `install.sh`

Do not do both. The two installs collide in two ways:

- your `~/.config/systemd/user/pergola.service` shadows the packaged unit, so
  `systemctl --user` keeps running the binary in `~/.local/bin`;
- both autostart entries (`~/.config/autostart` and `/usr/share/xdg/autostart`)
  launch at login. The daemon allows one per display, so the second exits with
  "a pergola daemon is already running" — harmless, but noise in the journal.

Remove the source install first (`systemctl --user disable --now pergola.service`
and delete `~/.local/bin/pergola*`), or keep the source install and skip the
package.

## Cutting a `pergola-bin` package

1. Tag a release and attach a binary (a CI workflow building on `x86_64` and
   `aarch64` is the missing piece).
2. Copy this `PKGBUILD`, rename `pkgname=pergola-bin`, drop `makedepends` and
   `build()`, and point `source=` at the release tarball with a real
   `sha256sums`.
3. Keep `package()` identical: the file list is what the acceptance test checks.

## Verifying the package without installing it

```sh
makepkg -f
bsdtar -tf pergola-git-*.pkg.tar.zst | sort
```

That lists exactly what the table above claims. `makepkg -si` additionally runs
`pergola doctor`-able binaries from `/usr/bin`, which is how the packaging was
verified when it was written.
