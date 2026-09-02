# Portal test plan

Harness: `scripts/run-portal-tests.py`  
Artifacts: `/tmp/omarchy-portal-test-report.md`, `/tmp/omarchy-portal-test-report.json`, `/tmp/omarchy-portal-test-shots/`

## Categories

| Category | What |
|----------|------|
| **env** | systemd units, D-Bus name, portals.conf routing, no user QML plugin, xdph picker, package vs git |
| **unit** | `cargo test --lib` |
| **api** | Non-UI or fire-and-forget portal calls via `portal-call.py` |
| **interactive** | Dialog opens (egui) → Esc cancel |

## Cases

### Environment

| ID | Expect |
|----|--------|
| env.xdg-desktop-portal-omarchy.service | active |
| env.xdg-desktop-portal.service | active |
| env.xdg-desktop-portal-hyprland.service | active |
| env.route.FileChooser | omarchy |
| env.route.Settings | omarchy |
| env.route.ScreenCast | hyprland |
| env.route.Secret | gnome-keyring |
| env.no_user_plugin | `~/.config/omarchy/plugins/omarchy-portal` is absent |
| env.xdph_picker | `~/.config/hypr/xdph.conf` present |
| env.dbus_name | `org.freedesktop.impl.portal.desktop.omarchy` owned |
| env.package_vs_git | informational |

### Unit

| ID | Expect |
|----|--------|
| unit.cargo | all lib tests pass |

### API

| ID | Expect |
|----|--------|
| api.settings | Read color-scheme + accent-color |
| api.notification | AddNotification succeeds |
| api.notification-remove | RemoveNotification succeeds |
| api.email | Compose with attachment FD / path |
| api.inhibit_seen | optional journal evidence |

### Interactive (open → Esc)

| ID | Portal |
|----|--------|
| ui.open | FileChooser Open (deep folder) |
| ui.save | FileChooser Save |
| ui.open-dir | FileChooser directory |
| ui.account | Account |
| ui.access | Access |
| ui.background | Background |
| ui.app-chooser | AppChooser / OpenURI ask |
| ui.screenshot | Screenshot |
| ui.dynamic-launcher | DynamicLauncher PrepareInstall |
| ui.pick-color | PickColor / hyprpicker path |
| ui.share_picker_bin | omarchy-share-picker present |


## Response codes

Frontend Request `response`: `0` success, `1` cancelled. Interactive Esc tests:

- most portals: `response` 0 or 1
- Access (impl): `reply[0] == 1` (deny/cancel)
- Background (impl): `result == 0` (Deny; Esc is not Allow once)
