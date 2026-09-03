# Packaging

The package ships system files only. Session routing is `omarchy-portals.conf`, picked when `XDG_CURRENT_DESKTOP` includes `Omarchy` (Omarchy sets `Omarchy:Hyprland`). D-Bus activates the backend; do not `systemctl --user enable` the unit.

Window rules and the ScreenCast picker belong in Omarchy (`default/hypr/apps/system.lua`, `config/hypr/xdph.conf`). This package does not write `~/.config/hypr`.

## System paths

| Component | Path |
|-----------|------|
| Portal daemon | `/usr/lib/xdg-desktop-portal-omarchy` |
| Portal descriptor | `/usr/share/xdg-desktop-portal/portals/omarchy.portal` |
| Portals.conf | `/usr/share/xdg-desktop-portal/omarchy-portals.conf` |
| D-Bus service | `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service` |
| systemd user unit | `/usr/lib/systemd/user/xdg-desktop-portal-omarchy.service` |

```bash
./scripts/install-user.sh              # user-local (dev)
sudo ./scripts/install-system.sh       # system
```

`xdg-desktop-portal-omarchy-setup` only reloads portal units. It does not copy `portals.conf` or edit Hyprland config.

AUR: [`aur/xdg-desktop-portal-omarchy-git/`](../aur/xdg-desktop-portal-omarchy-git/). Official Omarchy builds should use a tagged `xdg-desktop-portal-omarchy` PKGBUILD in `omarchy-pkgs`, same layout as `omacalc`. After that package is in `[omarchy]`, add it to `install/omarchy-base.packages` next to `xdg-desktop-portal-gtk` and `xdg-desktop-portal-hyprland`. Do not add the name before the package exists — ISO pacstrap and `omarchy-reinstall-pkgs` read that list.
