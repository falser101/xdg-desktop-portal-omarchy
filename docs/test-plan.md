# Portal test plan

Harness: `scripts/run-portal-tests.py`  
Artifacts: `/tmp/omarchy-portal-test-report.md`, `/tmp/omarchy-portal-test-report.json`, `/tmp/omarchy-portal-test-shots/`

## Categories

| Category | What |
|----------|------|
| **env** | systemd unit present, D-Bus name, portals.conf routing, no user QML plugin, package vs git |
| **unit** | `cargo test --lib` |
| **api** | Non-UI or fire-and-forget portal calls via `portal-call.py` |
| **interactive** | Dialog opens (egui) → Esc cancel |

## Cases

### Environment

| ID | Expect |
|----|--------|
| env.xdg-desktop-portal-omarchy.service | unit loaded (D-Bus activatable; need not be enabled) |
| env.xdg-desktop-portal.service | active |
| env.xdg-desktop-portal-hyprland.service | active |
| env.desktop | `XDG_CURRENT_DESKTOP` contains Omarchy |
| env.route.FileChooser | omarchy |
| env.route.Settings | omarchy |
| env.route.Screenshot | omarchy |
| env.route.ScreenCast | hyprland |
| env.route.Print | gtk |
| env.route.Secret | gnome-keyring |
| env.no_user_plugin | `~/.config/omarchy/plugins/omarchy-portal` is absent |
| env.dbus_name | `org.freedesktop.impl.portal.desktop.omarchy` owned (after activation) |
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


## Response codes

Frontend Request `response`: `0` success, `1` cancelled. Interactive Esc tests:

- most portals: `response` 0 or 1
- Access (impl): `reply[0] == 1` (deny/cancel)
- Background (impl): `result == 0` (Deny; Esc is not Allow once)
