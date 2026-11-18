# AUR packaging

`PKGBUILD` builds Pergola from source and follows `main`. It is the same code
`install.sh` installs, moved from your home directory to system paths.

## `pergola-git` and `pergola-bin`

AUR naming is a contract: `-bin` means *prebuilt binary, no compilation*, and
`-git` means *builds from the VCS branch*. `pergola-git` is the source package
and follows `main`.

`pergola-bin` is generated from a **stable release**. `docs/VERSIONING.md` says
dev builds are commits, not tags, so no release tag exists while the project is
`0.x.y-dev` — and therefore no `pergola-bin` package can honestly exist yet.
The pipeline that will produce it is in place:

* `.github/workflows/release.yml` — on a `v<version>` tag (matching `Cargo.toml`,
  never a `-dev` version) it builds `x86_64` and `aarch64`, packages them with
  `scripts/make-release-artifacts.sh`, verifies the checksums and attaches the
  tarballs, their `.sha256` files and the generated AUR files to the release.
  `workflow_dispatch` builds the same artifacts without publishing, so the
  pipeline can be exercised before a tag exists.
* `scripts/gen-bin-pkgbuild.sh <version> <x86-tarball> <aarch64-tarball> <dir>`
  writes `PKGBUILD` and `pergola.install` with the real `sha256sums`. Its
  `package()` is the `pergola-git` one minus the build, so the file list is
  identical (verified by building both packages and diffing `bsdtar -tf`).

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
| `$XDG_DATA_HOME/applications/pergola-actions.desktop` | `/usr/share/applications/pergola-actions.desktop` |
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

1. Tag a **stable** release whose version equals `Cargo.toml`
   (`git tag -s v1.0.0 && git push origin v1.0.0`). The workflow refuses a
   mismatch, and refuses any `-dev` version.
2. The release carries `pergola-<version>-{x86_64,aarch64}-linux-gnu.tar.gz`,
   their `.sha256` files, `PKGBUILD` and `pergola.install`.
3. Copy those two files into the AUR `pergola-bin` repo (`pergola.install` is
   `pergola-git`'s scriptlet with the package name substituted) and generate
   `.SRCINFO` with `makepkg --printsrcinfo > .SRCINFO`.

## Exercising the pipeline without a tag

The same scripts run by hand, so every step short of the tag is testable:

```sh
# 1. build the binary, then the artifact (the binary's --version must match)
cargo build --release
scripts/make-release-artifacts.sh 1.0.0 x86_64 target/release/pergola /tmp/dist
(cd /tmp/dist && sha256sum -c ./*.sha256)

# 2. generate the AUR files (pass the aarch64 tarball too; CI has both)
scripts/gen-bin-pkgbuild.sh 1.0.0 \
  /tmp/dist/pergola-1.0.0-x86_64-linux-gnu.tar.gz \
  /tmp/dist/pergola-1.0.0-aarch64-linux-gnu.tar.gz /tmp/aur

# 3. build the package. Point the arch's source at the local tarball: every
#    checksum is real, only the download is skipped.
cd /tmp/aur
sed -i 's|^source_x86_64=.*|source_x86_64=("file:///tmp/dist/pergola-1.0.0-x86_64-linux-gnu.tar.gz")|' PKGBUILD
makepkg -f
bsdtar -tf pergola-bin-*.pkg.tar.zst | sort
```

Step 3 is how the `package()` function and the file list were verified; the
sums in the generated `PKGBUILD` are the ones `sha256sum` computes over the
artifacts, not placeholders.

## Verifying the package without installing it

```sh
makepkg -f
bsdtar -tf pergola-git-*.pkg.tar.zst | sort
```

That lists exactly what the table above claims. `makepkg -si` additionally runs
`pergola doctor`-able binaries from `/usr/bin`, which is how the packaging was
verified when it was written.
