# ScreenCast

[中文](ScreenCast.md)

Status: **delegated capture + Omarchy preview picker**  
Sources: `scripts/omarchy-share-picker`, `shell/omarchy.portal/SharePickerDialog.qml`, `src/bin/omarchy_portal_capture.rs`  
Routing: `omarchy-portals.conf` / `~/.config/hypr/xdph.conf` (`custom_picker_binary`)

## Done

- Capture delegated to `xdg-desktop-portal-hyprland` (PipeWire / session / restore)
- Custom share picker (`custom_picker_binary` = `omarchy-share-picker`), Omarchy layout:

### UI layout

| Area | Content |
|------|---------|
| **Top bar** | **Display / Windows / Region** pages; monitor chips on Windows when multi-monitor; search on the right |
| **Display** | Whole-screen cards: `Quickshell.screens` as `ScreencopyView` source (same path as window-preview); name + resolution |
| **Windows** | Window grid (icon + title + toplevel preview) |
| **Region** | Own page: copy + `Select region` (`omarchy-capture-region`) |
| **Scrollbar** | Right-edge gutter (does not overlay previews); draggable + mouse wheel |
| **Footer** | Left: restore checkbox; right: Cancel / Share (same row) |

### Interaction

- Display page first; first monitor selected by default
- ↑↓←→ move selection on the current page; Enter / **Share** confirm; click to select, double-click to confirm; Esc cancel
- Footer checkbox:  
  **Allow the application to do this without asking next time** (restore token)
- Region: `omarchy-capture-region smart` (freeze + snap); full-monitor snap → `screen:NAME`; free rect → `region:OUT@x,y,w,h`

### Thumbnails & data

- Displays: Quickshell `ScreencopyView` + `ShellScreen` (`Quickshell.screens`, same path as Omarchy window-preview)
- Windows: `ScreencopyView` + `Hyprland.toplevels` (`hyprland_toplevel_export_v1`)
- Filters out the `Omarchy Portal` dialog itself
- stdout: `[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`
- Window IDs from `XDPH_WINDOW_SHARING_LIST`; thumbnails use `hyprctl` addresses

## Previews (open first, paint after)

- Dialog opens immediately; cards use Quickshell `ScreencopyView` (still frame + `captureFrame`, same as window-preview)
- Displays pass `Quickshell.screens` `ShellScreen` objects straight to `captureSource` — no name re-lookup, no PNG prefetch
- No grim prefetch (that used to hang OBS / block xdph)

## Deferred

- Virtual screen / Workspace synthetic outputs
- Custom region overlay (currently uses stock `omarchy-capture-region`)
- No PipeWire capture engine reimplemented in this daemon
- Viewport throttling / reduce monitor “hall of mirrors”

## Self-test

```bash
# Confirm xdph points at the Omarchy picker
grep custom_picker_binary ~/.config/hypr/xdph.conf

# Reinstall and restart
./scripts/install-user.sh
systemctl --user restart xdg-desktop-portal-omarchy xdg-desktop-portal xdg-desktop-portal-hyprland
omarchy restart shell

# OBS / browser “Share screen” should open the Omarchy card picker
```
