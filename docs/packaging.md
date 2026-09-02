# Packaging

## System paths

| Component | Path |
|-----------|------|
| Portal daemon | `/usr/lib/xdg-desktop-portal-omarchy` |
| User setup | `/usr/bin/xdg-desktop-portal-omarchy-setup` |
| Portal descriptor | `/usr/share/xdg-desktop-portal/portals/omarchy.portal` |
| Portals.conf template | `/usr/share/xdg-desktop-portal/omarchy-portals.conf` |
| D-Bus service | `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service` |
| systemd user unit | `/usr/lib/systemd/user/xdg-desktop-portal-omarchy.service` |

Session wiring (`hyprland-portals.conf`, `xdph.conf`) is not done by the package. Each user runs `xdg-desktop-portal-omarchy-setup`.

```bash
./scripts/install-user.sh              # user-local (dev)
sudo ./scripts/install-system.sh       # system
xdg-desktop-portal-omarchy-setup
```

AUR: [`aur/xdg-desktop-portal-omarchy-git/`](../aur/xdg-desktop-portal-omarchy-git/).
