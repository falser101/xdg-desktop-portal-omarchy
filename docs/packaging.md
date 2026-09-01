# Packaging & AUR

## Paths (system package)

| Component | Path |
|-----------|------|
| Portal daemon | `/usr/lib/xdg-desktop-portal-omarchy` |
| Window capture helper | `/usr/lib/omarchy-portal-capture` |
| Share picker | `/usr/bin/omarchy-share-picker` |
| User setup helper | `/usr/bin/xdg-desktop-portal-omarchy-setup` |
| Portal descriptor | `/usr/share/xdg-desktop-portal/portals/omarchy.portal` |
| Portals.conf template | `/usr/share/xdg-desktop-portal/omarchy-portals.conf` |
| D-Bus service | `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service` |
| systemd user unit | `/usr/lib/systemd/user/xdg-desktop-portal-omarchy.service` |
| QML plugin (packaged) | `/usr/share/xdg-desktop-portal-omarchy/omarchy.portal/` |

User session wiring (plugin copy, `xdph.conf`, portals.conf) is **not** done by the package install. Run:

```bash
xdg-desktop-portal-omarchy-setup
```

## Build / install from a checkout

```bash
# User-local (dev)
./scripts/install-user.sh

# System-wide (needs root, or DESTDIR for packaging)
sudo ./scripts/install-system.sh
# or
make install-system PREFIX=/usr

# After system install, each user:
xdg-desktop-portal-omarchy-setup
```

## Publish `xdg-desktop-portal-omarchy-git` to the AUR

### 1. Prerequisites

- AUR account + SSH key uploaded at https://aur.archlinux.org/account/
- Local Arch (or `makepkg` in a clean chroot) to test

```bash
ssh-keyscan aur.archlinux.org >> ~/.ssh/known_hosts
```

### 2. Test the PKGBUILD locally

```bash
cd aur/xdg-desktop-portal-omarchy-git
# Optional: point source at a local path while iterating — for real publish keep git+https
makepkg -si
xdg-desktop-portal-omarchy-setup
```

Regenerate `.SRCINFO` after editing `PKGBUILD`:

```bash
make aur-srcinfo
# or: cd aur/xdg-desktop-portal-omarchy-git && makepkg --printsrcinfo > .SRCINFO
```

### 3. First push to AUR

```bash
git clone ssh://aur@aur.archlinux.org/xdg-desktop-portal-omarchy-git.git
cd xdg-desktop-portal-omarchy-git
cp /path/to/xdg-desktop-portal-omarchy/aur/xdg-desktop-portal-omarchy-git/PKGBUILD .
cp /path/to/xdg-desktop-portal-omarchy/aur/xdg-desktop-portal-omarchy-git/xdg-desktop-portal-omarchy-git.install .
cp /path/to/xdg-desktop-portal-omarchy/aur/xdg-desktop-portal-omarchy-git/.SRCINFO .
git add PKGBUILD .SRCINFO xdg-desktop-portal-omarchy-git.install
git commit -m "Initial import: xdg-desktop-portal-omarchy-git"
git push -u origin master
```

### 4. Later updates

When upstream `main` moves, bump is automatic via `pkgver()` on the next `makepkg`. Maintainers typically:

```bash
cd xdg-desktop-portal-omarchy-git   # AUR clone
# refresh PKGBUILD from upstream repo if install script paths changed
makepkg --printsrcinfo > .SRCINFO
git add -u
git commit -m "Update to latest git"
git push
```

Users install with:

```bash
yay -S xdg-desktop-portal-omarchy-git
xdg-desktop-portal-omarchy-setup
```

## Notes

- The git package clones GitHub; `third_party/hyprland-preview-share-picker-lib` is vendored in-tree (no nested git fetch).
- `cargo build --frozen` requires `Cargo.lock` in the repo (already committed).
- Do not install into `$HOME` from `package()` — that breaks packaging and multi-user systems.
