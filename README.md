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

## Interfaces

| Interface | Implementation |
|-----------|----------------|
| FileChooser | Native file browser (Open / Save / SaveFiles, filters, Places) |
| Settings | Reads `~/.local/state/omarchy/current/theme/colors.toml` and emits `SettingChanged` |
| AppChooser | Desktop-file picker with icons |
| Account | Confirm + passwd / `~/.face` |
| Access | Grant / deny dialog |
| Notification | `omarchy-notification-send` |
| Inhibit | `systemd-inhibit` plus `CreateMonitor` / lock via Hyprland |
| Email | `xdg-email` |
| Wallpaper | `omarchy-theme-bg-set` |
| Screenshot | Confirm dialog + `grim` / `slurp` / `hyprpicker` |
| Background | Confirm + `~/.config/autostart` |
| DynamicLauncher | Confirm install; software-center token allowlist |
| Lockdown | Defaults matching a desktop session |
| Secret | Routed to `gnome-keyring` |

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
