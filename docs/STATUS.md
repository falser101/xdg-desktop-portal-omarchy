# Implementation status

[中文](STATUS.zh-CN.md)

Apps (Firefox, Chromium, Flatpak, …) talk only to the frontend
`xdg-desktop-portal`. This repo implements the backend
`org.freedesktop.impl.portal.desktop.omarchy`. On Hyprland the effective routing
file is:

```
~/.config/xdg-desktop-portal/hyprland-portals.conf
```

Source: `data/omarchy-portals.conf`.

Per-interface **done / deferred** notes live under
[`docs/portals/`](portals/). Edit only the file for the portal you change.

## Architecture

| Layer | Role |
|-------|------|
| Rust daemon `xdg-desktop-portal-omarchy` | D-Bus backend: export `org.freedesktop.impl.portal.*` |
| Quickshell plugin `omarchy-portal` | Dialog UI (`~/.config/omarchy/plugins/omarchy-portal/`) |
| `omarchy-share-picker` | Custom share picker for hyprland ScreenCast |
| `omarchy-portal-capture` | Window thumbnails via `hyprland_toplevel_export_v1` |
| egui `--picker` subprocess | Fallback when the shell plugin is unavailable |

Dialogs prefer `omarchy-shell shell summon omarchy-portal`. Windows are Quickshell
`FloatingWindow` (centered card), not fullscreen layer-shell.

```
App
  → xdg-desktop-portal (frontend)
    → omarchy (this repo)
    → hyprland (ScreenCast / GlobalShortcuts / InputCapture)
    → gnome-keyring (Secret)
    → gtk (fallback)
```

## Matrix

| Interface | Status | Doc |
|-----------|--------|-----|
| FileChooser | done | [portals/FileChooser.md](portals/FileChooser.md) |
| Settings | done | [portals/Settings.md](portals/Settings.md) |
| AppChooser | done (set default → mimeapps) | [portals/AppChooser.md](portals/AppChooser.md) |
| Account | done | [portals/Account.md](portals/Account.md) |
| Access | done (choices / icon) | [portals/Access.md](portals/Access.md) |
| Notification | done (bridge → FDO; actions / icon / persistent) | [portals/Notification.md](portals/Notification.md) |
| Inhibit | done | [portals/Inhibit.md](portals/Inhibit.md) |
| Email | done | [portals/Email.md](portals/Email.md) |
| Wallpaper | done | [portals/Wallpaper.md](portals/Wallpaper.md) |
| Lockdown | stub | [portals/Lockdown.md](portals/Lockdown.md) |
| Screenshot | done | [portals/Screenshot.md](portals/Screenshot.md) |
| Background | done (Allow / Allow once / Forbid + state signal) | [portals/Background.md](portals/Background.md) |
| DynamicLauncher | done | [portals/DynamicLauncher.md](portals/DynamicLauncher.md) |
| ScreenCast | delegated + Omarchy preview picker | [ScreenCast.en.md](portals/ScreenCast.en.md) · [中文](portals/ScreenCast.md) |
| GlobalShortcuts | delegated | [portals/GlobalShortcuts.md](portals/GlobalShortcuts.md) |
| InputCapture | delegated | [portals/InputCapture.md](portals/InputCapture.md) |
| Secret | delegated | [portals/Secret.md](portals/Secret.md) |
| Print | not implemented | [portals/Print.md](portals/Print.md) |
| RemoteDesktop | not implemented | [portals/RemoteDesktop.md](portals/RemoteDesktop.md) |
| Clipboard | not implemented | [portals/Clipboard.md](portals/Clipboard.md) |
| Usb | not implemented | [portals/Usb.md](portals/Usb.md) |

Location, Camera, Trash, NetworkMonitor, etc. exist on the frontend but are
rarely implemented by desktops; no dedicated notes unless an app gets stuck.

## Cross-portal deferred

Shared by most dialogs:

- **`parent_window` + `modal`:** currently a standalone `FloatingWindow`, not attached to the caller.

FileChooser-only deferred items: [portals/FileChooser.md](portals/FileChooser.md)
(sandbox path restore, list semantics, …).

## Self-test

```bash
python3 scripts/portal-call.py settings
python3 scripts/portal-call.py open
python3 scripts/portal-call.py save
python3 scripts/portal-call.py open-dir
python3 scripts/portal-call.py account
cargo run -- --demo account
python3 scripts/portal-call.py notification
python3 scripts/portal-call.py notification-remove
python3 scripts/portal-call.py open-uri
python3 scripts/portal-call.py screenshot
python3 scripts/portal-call.py pick-color
python3 scripts/portal-call.py background

cargo run -- --demo file-chooser
cargo run -- --demo access
```

Install: `./scripts/install-user.sh`.  
After plugin edits (if `keepLoaded` does not hot-reload): `omarchy restart shell`.
