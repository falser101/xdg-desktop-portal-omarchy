# xdg-desktop-portal-omarchy

[中文](README.zh-CN.md)

[xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) backend for Omarchy (Hyprland). Dialogs use the built-in egui picker.

Omarchy sets `XDG_CURRENT_DESKTOP=Omarchy:Hyprland`. xdg-desktop-portal then loads `/usr/share/xdg-desktop-portal/omarchy-portals.conf` and D-Bus-activates this backend. No per-user setup.

## Implemented

| Portal | What it does |
|--------|----------------|
| **FileChooser** | Open / Save / SaveFiles, image thumbnails |
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

ScreenCast / GlobalShortcuts / InputCapture stay on `xdg-desktop-portal-hyprland`. Print stays on `xdg-desktop-portal-gtk`. Secret stays on `gnome-keyring`.

## Install

Packaged with Omarchy, or:

```bash
yay -S xdg-desktop-portal-omarchy-git
```

From a checkout: `./scripts/install-user.sh` or `sudo ./scripts/install-system.sh`. Reload Hyprland or re-login so `XDG_CURRENT_DESKTOP` includes Omarchy.

Packaging: [docs/packaging.md](docs/packaging.md).

## License

MIT
