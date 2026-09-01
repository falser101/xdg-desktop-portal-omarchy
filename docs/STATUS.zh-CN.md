# 实现状态

[English](STATUS.md)

应用（Firefox、Chromium、Flatpak 等）只跟前端 `xdg-desktop-portal` 说话。本仓库实现的是后端 `org.freedesktop.impl.portal.desktop.omarchy`。Hyprland 上实际生效的路由文件是：

```
~/.config/xdg-desktop-portal/hyprland-portals.conf
```

对应源文件：`data/omarchy-portals.conf`。

每个接口的**已完成 / 延后 / 对照 KDE**细节单独记在 [`docs/portals/`](portals/) 下，改某一 portal 时只改对应文件。

## 架构

| 层 | 作用 |
|----|------|
| Rust daemon `xdg-desktop-portal-omarchy` | D-Bus 后端：导出 `org.freedesktop.impl.portal.*` |
| Quickshell 插件 `omarchy-portal` | 对话框 UI（装在 `~/.config/omarchy/plugins/omarchy-portal/`） |
| `omarchy-share-picker` | 给 hyprland ScreenCast 用的自定义分享选择器 |
| `omarchy-portal-capture` | 窗口缩略图（`hyprland_toplevel_export_v1`） |
| egui `--picker` 子进程 | shell 插件不可用时的后备对话框 |

对话框优先 `omarchy-shell shell summon omarchy-portal`。窗口是 Quickshell `FloatingWindow`（居中卡片），不是全屏 layer-shell。

```
应用
  → xdg-desktop-portal（前端）
    → omarchy（本仓库）
    → hyprland（ScreenCast / GlobalShortcuts / InputCapture）
    → gnome-keyring（Secret）
    → gtk（兜底）
```

## 对照表

| 接口 | 状态 | 文档 |
|------|------|------|
| FileChooser | 已实现 | [portals/FileChooser.md](portals/FileChooser.md) |
| Settings | 已实现 | [portals/Settings.md](portals/Settings.md) |
| AppChooser | 已实现（含设为默认 → mimeapps） | [portals/AppChooser.md](portals/AppChooser.md) |
| Account | 已实现（对齐 KDE UserInfoDialog） | [portals/Account.md](portals/Account.md) |
| Access | 已实现（choices / icon） | [portals/Access.md](portals/Access.md) |
| Notification | 已实现（薄桥 → FDO；含 action / 图标 / 常驻） | [portals/Notification.md](portals/Notification.md) |
| Inhibit | 已实现 | [portals/Inhibit.md](portals/Inhibit.md) |
| Email | 已实现 | [portals/Email.md](portals/Email.md) |
| Wallpaper | 已实现 | [portals/Wallpaper.md](portals/Wallpaper.md) |
| Lockdown | 桩 | [portals/Lockdown.md](portals/Lockdown.md) |
| Screenshot | 已实现 | [portals/Screenshot.md](portals/Screenshot.md) |
| Background | 已实现（Allow/Allow once/Forbid + 状态信号） | [portals/Background.md](portals/Background.md) |
| DynamicLauncher | 已实现 | [portals/DynamicLauncher.md](portals/DynamicLauncher.md) |
| ScreenCast | 委托采集 + Omarchy 预览选择器 | [ScreenCast.md](portals/ScreenCast.md) · [EN](portals/ScreenCast.en.md) |
| GlobalShortcuts | 委托 | [portals/GlobalShortcuts.md](portals/GlobalShortcuts.md) |
| InputCapture | 委托 | [portals/InputCapture.md](portals/InputCapture.md) |
| Secret | 委托 | [portals/Secret.md](portals/Secret.md) |
| Print | 未实现 | [portals/Print.md](portals/Print.md) |
| RemoteDesktop | 未实现 | [portals/RemoteDesktop.md](portals/RemoteDesktop.md) |
| Clipboard | 未实现 | [portals/Clipboard.md](portals/Clipboard.md) |
| Usb | 未实现 | [portals/Usb.md](portals/Usb.md) |

Location、Camera、Trash、NetworkMonitor 等前端有、常见桌面很少自实现的 impl：没有单独文档，除非有应用卡住再补。

## 优先延后项（跨 portal）

多数对话框共用：

- **`parent_window` + `modal`**：KDE 会附着调用方窗口；Omarchy 当前独立 `FloatingWindow`。

FileChooser 专属延后见 [portals/FileChooser.md](portals/FileChooser.md)（沙箱路径还原、KIO、列表语义等）。

## 自测

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

安装：`./scripts/install-user.sh`。插件改完若 keepLoaded 没热更新：`omarchy restart shell`。
