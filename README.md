# xdg-desktop-portal-omarchy

Omarchy backend for [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/).

This is not a GTK theme. It implements `org.freedesktop.impl.portal.*` on
`org.freedesktop.impl.portal.desktop.omarchy` and is what Firefox, Chromium,
Flatpak, and host apps talk to for file pickers, appearance settings, and
related desktop integration.

ScreenCast / GlobalShortcuts / InputCapture stay on
`xdg-desktop-portal-hyprland` (PipeWire / Hyprland protocols). The share
picker itself is Omarchy-styled (`omarchy-share-picker`). Screenshot,
FileChooser, and the rest of the GTK half are this backend.

**Implemented vs not:** [docs/STATUS.md](docs/STATUS.md)（总表）+ [docs/portals/](docs/portals/)（每个 portal 单独记录已完成与延后项，含对照 KDE）。

## Interfaces

| Interface | Status | Notes |
|-----------|--------|-------|
| FileChooser | done | [docs/portals/FileChooser.md](docs/portals/FileChooser.md) — Open/Save/Recent/choices；延后 parent_window / 沙箱路径 / KIO |
| Settings | done | [docs/portals/Settings.md](docs/portals/Settings.md) |
| AppChooser | done | [docs/portals/AppChooser.md](docs/portals/AppChooser.md) |
| Account | done | [docs/portals/Account.md](docs/portals/Account.md) |
| Access | done | [docs/portals/Access.md](docs/portals/Access.md) |
| Notification | done | [docs/portals/Notification.md](docs/portals/Notification.md) |
| Inhibit | done | [docs/portals/Inhibit.md](docs/portals/Inhibit.md) |
| Email | done | [docs/portals/Email.md](docs/portals/Email.md) |
| Wallpaper | done | [docs/portals/Wallpaper.md](docs/portals/Wallpaper.md) |
| Screenshot | done | [docs/portals/Screenshot.md](docs/portals/Screenshot.md) |
| Background | done | [docs/portals/Background.md](docs/portals/Background.md) |
| DynamicLauncher | done | [docs/portals/DynamicLauncher.md](docs/portals/DynamicLauncher.md) |
| Lockdown | stub | [docs/portals/Lockdown.md](docs/portals/Lockdown.md) |
| ScreenCast | delegated | [docs/portals/ScreenCast.md](docs/portals/ScreenCast.md) |
| GlobalShortcuts | delegated | [docs/portals/GlobalShortcuts.md](docs/portals/GlobalShortcuts.md) |
| InputCapture | delegated | [docs/portals/InputCapture.md](docs/portals/InputCapture.md) |
| Secret | delegated | [docs/portals/Secret.md](docs/portals/Secret.md) |
| Print | **not implemented** | [docs/portals/Print.md](docs/portals/Print.md) |
| RemoteDesktop | **not implemented** | [docs/portals/RemoteDesktop.md](docs/portals/RemoteDesktop.md) |
| Clipboard / Usb | **not implemented** | [Clipboard](docs/portals/Clipboard.md) / [Usb](docs/portals/Usb.md) |

## Build

```bash
cargo build --release
./scripts/install-user.sh
```

User install writes:

- `~/.local/libexec/xdg-desktop-portal-omarchy`
- `~/.local/share/xdg-desktop-portal/portals/omarchy.portal`
- `~/.config/xdg-desktop-portal/hyprland-portals.conf`
- `~/.config/systemd/user/xdg-desktop-portal-omarchy.service`

Then restart `xdg-desktop-portal`. `XDG_CURRENT_DESKTOP=Hyprland` on Omarchy,
so the Hyprland portals.conf is the file that is consulted.

## Demo (no D-Bus)

```bash
cargo run -- --demo file-chooser
```

## Preferred config

```ini
[preferred]
default=omarchy;hyprland;gtk
org.freedesktop.impl.portal.ScreenCast=hyprland
org.freedesktop.impl.portal.GlobalShortcuts=hyprland
org.freedesktop.impl.portal.InputCapture=hyprland
org.freedesktop.impl.portal.Screenshot=omarchy
org.freedesktop.impl.portal.Secret=gnome-keyring
```
