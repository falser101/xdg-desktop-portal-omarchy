# ScreenCast

[中文](ScreenCast.md)

Status: **delegated capture + Omarchy preview picker**  
Reference: KDE `ScreenChooserDialog` (KWin live PipeWire preview); capture on Hyprland  
Sources: `scripts/omarchy-share-picker`, `shell/omarchy.portal/SharePickerDialog.qml`, `src/bin/omarchy_portal_capture.rs`  
Routing: `omarchy-portals.conf` / `~/.config/hypr/xdph.conf` (`custom_picker_binary`)

## Done

- Capture delegated to `xdg-desktop-portal-hyprland` (PipeWire / session / restore)
- Custom share picker (`custom_picker_binary` = `omarchy-share-picker`), layout aligned with KDE:

### UI layout

| Area | Content |
|------|---------|
| **Top bar** | `Share region` on the left; monitor chips (geometry filter) when multi-monitor; search on the right |
| **Body** | Displays grid → “Windows” separator → Windows grid (single scroll view) |
| **Columns** | Responsive to dialog width (~≥260px per tile, 1–6 columns); keyboard grid matches |
| **Cards** | Displays and Windows share the **same** card chrome and cell width (icon + title header + preview); no double-layer preview background |
| **Scrollbar** | Right-edge gutter (does not overlay previews); draggable + mouse wheel |
| **Footer** | Left: restore checkbox (KDE wording); right: Cancel / Share (same row) |

### Interaction

- First item selected by default
- ↑↓←→ move selection across Displays / Windows
- Enter / **Share** confirm; click to select, double-click to confirm; Esc cancel
- Footer checkbox text matches KDE:  
  **Allow the application to do this without asking next time** (restore token)
- `Share region`: `omarchy-capture-region smart` (freeze + snap); full-monitor snap → `screen:NAME`; free rect → `region:OUT@x,y,w,h`

### Thumbnails & data

- Displays: `grim -o`
- Windows: `omarchy-portal-capture` → `hyprland_toplevel_export_v1` (no bleed across overlapping windows)
- Filters out the `Omarchy Portal` dialog itself
- stdout: `[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`
- Window IDs from `XDPH_WINDOW_SHARING_LIST`; thumbnails use `hyprctl` addresses

## Previews (KDE-style: open first, paint after)

- Dialog opens immediately; cards use Quickshell `ScreencopyView` (`live` while hovered/selected)
- Displays → `Quickshell.screens`; windows → `Hyprland.toplevels` by address
- **Not** KDE `PipeWireSourceItem` — Hyprland lacks `zkde_screencast_unstable_v1`
- No grim / `omarchy-portal-capture` PNG prefetch (that used to hang OBS / block xdph)

## Deferred

- Virtual screen / Workspace synthetic outputs (KDE `OutputsModel` options)
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
