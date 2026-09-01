# Packaging brief for Omarchy maintainers (Path B)

How this would land like `hyprland-preview-share-picker`: **external upstream + PKGBUILD in `omarchy-pkgs` + small wiring in `basecamp/omarchy`**.

## Upstream

| | |
|--|--|
| Source | https://github.com/falser101/xdg-desktop-portal-omarchy |
| License | MIT |
| Build | `cargo build --release --frozen` (+ `Cargo.lock` in tree) |
| UI | Quickshell plugin under `shell/omarchy.portal/` |
| Capture helper | `omarchy-portal-capture` (toplevel-export for window thumbs) |
| Reference PKGBUILD | `aur/xdg-desktop-portal-omarchy-git/PKGBUILD` in this repo |

Vendored: `third_party/hyprland-preview-share-picker-lib` (no nested git fetch at build time).

## Package name suggestion

- **`xdg-desktop-portal-omarchy`** — pinned commit or version tag (preferred for Omarchy repo)
- Keep community **`-git`** on AUR for nightlies if useful

## Files the package should install

Same as `docs/packaging.md`:

| Component | Path |
|-----------|------|
| Daemon | `/usr/lib/xdg-desktop-portal-omarchy` |
| Capture helper | `/usr/lib/omarchy-portal-capture` |
| Share picker | `/usr/bin/omarchy-share-picker` |
| Setup helper | `/usr/bin/xdg-desktop-portal-omarchy-setup` |
| Portal descriptor | `/usr/share/xdg-desktop-portal/portals/omarchy.portal` |
| Portals.conf template | `/usr/share/xdg-desktop-portal/omarchy-portals.conf` |
| D-Bus service | `/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service` |
| systemd user unit | `/usr/lib/systemd/user/xdg-desktop-portal-omarchy.service` |
| QML plugin | `/usr/share/xdg-desktop-portal-omarchy/omarchy.portal/` |

`package()` must not write `$HOME`. Session wiring stays in `xdg-desktop-portal-omarchy-setup` or an Omarchy migration / finalize step.

## Runtime dependencies (high level)

- `xdg-desktop-portal`, `xdg-desktop-portal-hyprland`
- Hyprland + grim (display thumbs) + Omarchy shell / Quickshell for dialogs
- Existing Omarchy capture helpers used by region share (`omarchy-capture-region`)

Exact `depends=` should be copied from the AUR PKGBUILD and adjusted to Omarchy package names.

## Suggested Omarchy wiring (later small PR)

### 1. Packages

Add to `install/omarchy-base.packages` **or** an optional install group first:

```text
xdg-desktop-portal-omarchy
```

Optionally demote / keep:

```text
xdg-desktop-portal-gtk          # fallback
hyprland-preview-share-picker   # until custom_picker switches
```

### 2. xdph share picker

`config/hypr/xdph.conf` (and skel / refresh path):

```ini
screencopy {
    allow_token_by_default = true
    custom_picker_binary = omarchy-share-picker
}
```

### 3. Portal preference

Ship or migrate user config so interactive portals prefer `omarchy` (see packaged `omarchy-portals.conf`):

```ini
[preferred]
default=omarchy;hyprland;gtk
org.freedesktop.impl.portal.ScreenCast=hyprland
org.freedesktop.impl.portal.GlobalShortcuts=hyprland
org.freedesktop.impl.portal.InputCapture=hyprland
org.freedesktop.impl.portal.FileChooser=omarchy
# ... other interactive interfaces = omarchy
org.freedesktop.impl.portal.Secret=gnome-keyring
```

Existing users need a **migration** (or `xdg-desktop-portal-omarchy-setup`) plus:

```bash
systemctl --user restart xdg-desktop-portal xdg-desktop-portal-hyprland xdg-desktop-portal-omarchy
```

### 4. Window rules

Stock already floats `xdg-desktop-portal-gtk`. Omarchy portal dialogs are Quickshell (`org.quickshell`, title `Omarchy Portal`) — confirm float/focus rules still look right under Omarchy defaults.

## Rollout recommendation

1. **Optional package only** — document `makepkg` / Omarchy pkg install + setup.
2. Dogfood on edge/RC.
3. Default wiring + migration once FileChooser / share-picker feedback is good.
4. Decide whether `hyprland-preview-share-picker` stays as fallback or is dropped from base.

## What we are not asking for

- Merging this Rust tree into `basecamp/omarchy`
- Replacing xdph capture
- Claiming Print / RemoteDesktop / Clipboard / Usb are done
