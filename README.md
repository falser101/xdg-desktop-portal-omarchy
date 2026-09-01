# xdg-desktop-portal-omarchy

[中文](README.zh-CN.md)

Omarchy backend for [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/).

This is not a GTK theme. It implements `org.freedesktop.impl.portal.*` on
`org.freedesktop.impl.portal.desktop.omarchy` and is what Firefox, Chromium,
Flatpak, and host apps talk to for file pickers, appearance settings, and
related desktop integration.

**Capture-heavy portals** (ScreenCast / GlobalShortcuts / InputCapture) stay on
`xdg-desktop-portal-hyprland` (PipeWire / Hyprland protocols). The **share
picker UI** is Omarchy-styled (`omarchy-share-picker` + Quickshell plugin).
Screenshot, FileChooser, Access, and the rest of the interactive half live in
this backend.

**Status index:** [docs/STATUS.md](docs/STATUS.md) (English) · [docs/STATUS.zh-CN.md](docs/STATUS.zh-CN.md) (中文)  
**Per-portal notes:** [docs/portals/](docs/portals/)

## Interfaces

| Interface | Status | Notes |
|-----------|--------|-------|
| FileChooser | done | [FileChooser](docs/portals/FileChooser.md) |
| Settings | done | [Settings](docs/portals/Settings.md) |
| AppChooser | done | [AppChooser](docs/portals/AppChooser.md) |
| Account | done (KDE UserInfoDialog) | [Account](docs/portals/Account.md) |
| Access | done | [Access](docs/portals/Access.md) |
| Notification | done (FDO bridge; actions / icon / persistent) | [Notification](docs/portals/Notification.md) |
| Inhibit | done | [Inhibit](docs/portals/Inhibit.md) |
| Email | done | [Email](docs/portals/Email.md) |
| Wallpaper | done | [Wallpaper](docs/portals/Wallpaper.md) |
| Screenshot | done | [Screenshot](docs/portals/Screenshot.md) |
| Background | done (Allow / Allow once / Forbid) | [Background](docs/portals/Background.md) |
| DynamicLauncher | done | [DynamicLauncher](docs/portals/DynamicLauncher.md) |
| Lockdown | stub | [Lockdown](docs/portals/Lockdown.md) |
| ScreenCast | delegated + Omarchy picker | [ScreenCast](docs/portals/ScreenCast.en.md) · [中文](docs/portals/ScreenCast.md) |
| GlobalShortcuts | delegated | [GlobalShortcuts](docs/portals/GlobalShortcuts.md) |
| InputCapture | delegated | [InputCapture](docs/portals/InputCapture.md) |
| Secret | delegated | [Secret](docs/portals/Secret.md) |
| Print | **not implemented** | [Print](docs/portals/Print.md) |
| RemoteDesktop | **not implemented** | [RemoteDesktop](docs/portals/RemoteDesktop.md) |
| Clipboard / Usb | **not implemented** | [Clipboard](docs/portals/Clipboard.md) / [Usb](docs/portals/Usb.md) |

## ScreenCast share picker (Omarchy)

Capture stays on Hyprland; UI is `scripts/omarchy-share-picker` →
`SharePickerDialog.qml`, registered as `custom_picker_binary` in `xdph.conf`.

- **Top bar:** `Share region` · monitor chips (multi-monitor filter) · search
- **Body:** Displays grid → “Windows” separator → Windows grid  
  Responsive columns (~≥260px per tile, 1–6). Same card chrome for displays and windows.
- **Thumbnails:** displays via `grim -o`; windows via `omarchy-portal-capture` (`hyprland_toplevel_export_v1`)
- **Selection:** first item selected by default; ↑↓←→ move; Enter / **Share** confirm; click select, double-click confirm
- **Footer (KDE-aligned):** left — *Allow the application to do this without asking next time*; right — Cancel / Share
- **Scrollbar:** right-edge gutter (does not overlay previews); mouse wheel works

Details: [docs/portals/ScreenCast.en.md](docs/portals/ScreenCast.en.md).

## Build & install

```bash
cargo build --release
./scripts/install-user.sh
```

User install writes:

- `~/.local/libexec/xdg-desktop-portal-omarchy`
- `~/.local/libexec/omarchy-portal-capture`
- `~/.local/bin/omarchy-share-picker`
- `~/.local/share/xdg-desktop-portal/portals/omarchy.portal`
- `~/.config/xdg-desktop-portal/hyprland-portals.conf`
- `~/.config/systemd/user/xdg-desktop-portal-omarchy.service`
- `~/.config/omarchy/plugins/omarchy-portal/` (Quickshell UI)

Then restart portals / shell if needed:

```bash
systemctl --user restart xdg-desktop-portal-omarchy xdg-desktop-portal xdg-desktop-portal-hyprland
omarchy restart shell
```

On Omarchy, `XDG_CURRENT_DESKTOP=Hyprland`, so `hyprland-portals.conf` is the file that is consulted.

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
