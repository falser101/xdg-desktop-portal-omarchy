# Discussion draft (basecamp/omarchy Suggestions)

Paste into: https://github.com/basecamp/omarchy/discussions/categories/suggestions

**Title:** Optional Omarchy-native xdg-desktop-portal backend (FileChooser / share picker), keep xdph for capture

---

## Summary

Proposal to package an **optional** Omarchy-styled `xdg-desktop-portal` backend as a **standalone package** (same pattern as `hyprland-preview-share-picker`), then wire it from `omarchy` / `omarchy-pkgs` when you are ready.

- **Upstream repo:** https://github.com/falser101/xdg-desktop-portal-omarchy
- **Local install today:** AUR-style `makepkg -si` from `aur/xdg-desktop-portal-omarchy-git/`
- **Not asking to merge the Rust tree into `basecamp/omarchy`.** Path B: keep this repo as upstream; official only packages + wires.

This also helps the FileChooser / GTK portal timeout class of bugs (e.g. [#7944](https://github.com/basecamp/omarchy/issues/7944)).

## What it does

| Layer | Who |
|-------|-----|
| FileChooser, Settings, AppChooser, Account, Access, Notification, Inhibit, Email, Wallpaper, Screenshot, Background, DynamicLauncher | `xdg-desktop-portal-omarchy` (Quickshell UI) |
| ScreenCast / GlobalShortcuts / InputCapture **capture** | still `xdg-desktop-portal-hyprland` |
| ScreenCast **share picker UI** | `omarchy-share-picker` via `custom_picker_binary` in `xdph.conf` |
| Secret | still `gnome-keyring` |

UI matches Omarchy shell theming (not GTK). Share picker: displays + windows, region, allow-once token, live window previews via `hyprland_toplevel_export_v1`.

## Why not a monorepo PR

`basecamp/omarchy` is packaging / shell / migrations. This backend is a separate Rust + Quickshell project (~6k LOC) with a capture helper and vendored protocol bits — same class of thing as packaging `hyprland-preview-share-picker` from an external URL, not embedding it in the Omarchy tree.

## Suggested official landing (Path B)

1. **Package** `xdg-desktop-portal-omarchy` in `omarchy-pkgs` from this GitHub URL (stable tag or commit pin).
2. **Optional first:** ship in the Omarchy repo package set without forcing every install.
3. **Small wiring PR** on `basecamp/omarchy` when you want it:
   - depend on / recommend the package
   - default or migration for `~/.config/xdg-desktop-portal/` portals preference (`omarchy` ahead of `gtk` for interactive portals)
   - `config/hypr/xdph.conf`: `custom_picker_binary = omarchy-share-picker`
4. Keep `xdg-desktop-portal-hyprland` for capture; keep GTK as fallback if desired.

Happy to adjust packaging layout to match `omarchy-pkgs` conventions.

## Try it (Arch / Omarchy)

```bash
git clone https://github.com/falser101/xdg-desktop-portal-omarchy.git
cd xdg-desktop-portal-omarchy/aur/xdg-desktop-portal-omarchy-git
makepkg -si
xdg-desktop-portal-omarchy-setup
systemctl --user restart xdg-desktop-portal xdg-desktop-portal-hyprland xdg-desktop-portal-omarchy
```

## Screenshots

Attach from `docs/upstream/assets/` in the repo (or the copies below after push):

| Asset | What |
|-------|------|
| `open.png` | FileChooser Open |
| `save.png` | FileChooser Save |
| `screenshot.png` | Screenshot portal |
| `access.png` | Access dialog |
| `app-chooser.png` | AppChooser |
| `share-picker.png` | Omarchy ScreenCast share picker (live previews) |

## Status / gaps (honest)

**Done enough for optional packaging:** FileChooser, Settings, AppChooser, Account, Access, Notification, Inhibit, Email, Wallpaper, Screenshot, Background, DynamicLauncher; ScreenCast picker delegated to xdph + Omarchy UI.

**Not implemented yet:** Print, RemoteDesktop, Clipboard, Usb.

**Still on other backends:** ScreenCast/GlobalShortcuts/InputCapture capture → hyprland; Secret → gnome-keyring.

## Ask

Would you take this as an **optional package** in the Omarchy package set (external upstream + PKGBUILD), with a later wiring PR — same model as `hyprland-preview-share-picker`?
