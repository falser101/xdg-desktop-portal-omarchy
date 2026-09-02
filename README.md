# xdg-desktop-portal-omarchy

[中文](README.zh-CN.md)

[xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) backend for Omarchy (Hyprland). Dialogs use the Omarchy Quickshell plugin.

## Implemented

| Portal | What it does |
|--------|----------------|
| **FileChooser** | Open / Save / SaveFiles — places, filters, collapsible breadcrumbs, search, preview, new folder, Documents path restore |
| **Settings** | Appearance / accent color for sandboxed apps |
| **AppChooser** | Open-with list; optional set-as-default |
| **Account** | User name / avatar dialog |
| **Access** | Permission prompt (choices, icon) |
| **Screenshot** | Interactive capture + pick-color |
| **Background** | Allow / Allow once / Forbid |
| **DynamicLauncher** | Install / uninstall web-app launchers |
| **Notification** | Bridge to Freedesktop notifications (actions, icon, persistent) |
| **Inhibit** | Session idle / sleep inhibit |
| **Email** | Compose via `mailto:` / configured client (attachments) |
| **Wallpaper** | Set desktop background |
| **Lockdown** | Stub |
| **ScreenCast UI** | Share picker — Display / Windows / Region pages with live `ScreencopyView` previews |

ScreenCast / GlobalShortcuts / InputCapture **capture** stays on `xdg-desktop-portal-hyprland`. Secret stays on `gnome-keyring`. This repo supplies the Omarchy-styled share picker (`omarchy-share-picker`).

## Install

```bash
yay -S xdg-desktop-portal-omarchy-git
xdg-desktop-portal-omarchy-setup
```

From a checkout:

```bash
./scripts/install-user.sh          # user-local (dev)
# or
sudo ./scripts/install-system.sh   # system-wide
xdg-desktop-portal-omarchy-setup
```

Packaging notes: [docs/packaging.md](docs/packaging.md).

## License

MIT
