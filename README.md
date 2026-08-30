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

**Implemented vs not:** see [docs/STATUS.md](docs/STATUS.md).

## Interfaces

| Interface | Status | Notes |
|-----------|--------|-------|
| FileChooser | done | Open / Save / Places / filters / new folder / overwrite |
| Settings | done | `colors.toml` + `SettingChanged` |
| AppChooser | done | Desktop-file picker with icons |
| Account | done | Confirm + passwd / `~/.face` |
| Access | done | Grant / deny dialog |
| Notification | done | `omarchy-notification-send` |
| Inhibit | done | `systemd-inhibit` + lock monitor |
| Email | done | `xdg-email` |
| Wallpaper | done | `omarchy-theme-bg-set` |
| Screenshot | done | Omarchy dialog + `grim` / `slurp` / `hyprpicker` |
| Background | done | Confirm + `~/.config/autostart` |
| DynamicLauncher | done | Confirm install; software-center token allowlist |
| Lockdown | stub | Defaults for a desktop session |
| ScreenCast | delegated | Hyprland capture; Omarchy share picker |
| GlobalShortcuts | delegated | Hyprland |
| InputCapture | delegated | Hyprland |
| Secret | delegated | `gnome-keyring` |
| Print | **not implemented** | Deferred |
| RemoteDesktop | **not implemented** | |
| Clipboard / Usb | **not implemented** | |

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
