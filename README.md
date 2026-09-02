# xdg-desktop-portal-omarchy

[中文](README.zh-CN.md)

[xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) backend for Omarchy (Hyprland). Dialogs use the built-in egui picker.

## Implemented

| Portal | What it does |
|--------|----------------|
| **FileChooser** | Open / Save / SaveFiles |
| **Settings** | Appearance / accent color |
| **AppChooser** | Open-with; optional set-as-default |
| **Account** | User name / avatar |
| **Access** | Permission prompt |
| **Screenshot** | Interactive capture + pick-color |
| **Background** | Allow / Allow once / Deny |
| **DynamicLauncher** | Install / uninstall web-app launchers |
| **Notification** | Freedesktop notification bridge |
| **Inhibit** | Idle / sleep inhibit |
| **Email** | Compose via `mailto:` (attachments) |
| **Wallpaper** | Set desktop background |
| **Lockdown** | Stub (`disable-location` on) |

ScreenCast / GlobalShortcuts / InputCapture stay on `xdg-desktop-portal-hyprland`. Secret stays on `gnome-keyring`.

## Install

```bash
yay -S xdg-desktop-portal-omarchy-git
xdg-desktop-portal-omarchy-setup
```

From a checkout: `./scripts/install-user.sh` or `sudo ./scripts/install-system.sh` then `xdg-desktop-portal-omarchy-setup`.

Packaging: [docs/packaging.md](docs/packaging.md).

## License

MIT
