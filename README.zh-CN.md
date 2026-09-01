# xdg-desktop-portal-omarchy

[English](README.md)

Omarchy 的 [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) 后端实现。

这不是 GTK 主题。它在 `org.freedesktop.impl.portal.desktop.omarchy` 上实现
`org.freedesktop.impl.portal.*`，供 Firefox、Chromium、Flatpak 以及本机应用调用
文件选择、外观设置等相关桌面集成能力。

**采集类 portal**（ScreenCast / GlobalShortcuts / InputCapture）仍走
`xdg-desktop-portal-hyprland`（PipeWire / Hyprland 协议）。**分享选择器 UI** 由
Omarchy 提供（`omarchy-share-picker` + Quickshell 插件）。Screenshot、FileChooser、
Access 等交互对话框由本仓库实现。

**状态总表：** [docs/STATUS.zh-CN.md](docs/STATUS.zh-CN.md)（中文）· [docs/STATUS.md](docs/STATUS.md)（English）  
**各 portal 笔记：** [docs/portals/](docs/portals/)

## 接口一览

| 接口 | 状态 | 说明 |
|------|------|------|
| FileChooser | 已实现 | [FileChooser](docs/portals/FileChooser.md) |
| Settings | 已实现 | [Settings](docs/portals/Settings.md) |
| AppChooser | 已实现 | [AppChooser](docs/portals/AppChooser.md) |
| Account | 已实现 | [Account](docs/portals/Account.md) |
| Access | 已实现 | [Access](docs/portals/Access.md) |
| Notification | 已实现 | [Notification](docs/portals/Notification.md) |
| Inhibit | 已实现 | [Inhibit](docs/portals/Inhibit.md) |
| Email | 已实现 | [Email](docs/portals/Email.md) |
| Wallpaper | 已实现 | [Wallpaper](docs/portals/Wallpaper.md) |
| Screenshot | 已实现 | [Screenshot](docs/portals/Screenshot.md) |
| Background | 已实现 | [Background](docs/portals/Background.md) |
| DynamicLauncher | 已实现 | [DynamicLauncher](docs/portals/DynamicLauncher.md) |
| Lockdown | 桩 | [Lockdown](docs/portals/Lockdown.md) |
| ScreenCast | 委托采集 + Omarchy 选择器 | [ScreenCast 中文](docs/portals/ScreenCast.md) · [EN](docs/portals/ScreenCast.en.md) |
| GlobalShortcuts | 委托 | [GlobalShortcuts](docs/portals/GlobalShortcuts.md) |
| InputCapture | 委托 | [InputCapture](docs/portals/InputCapture.md) |
| Secret | 委托 | [Secret](docs/portals/Secret.md) |
| Print | **未实现** | [Print](docs/portals/Print.md) |
| RemoteDesktop | **未实现** | [RemoteDesktop](docs/portals/RemoteDesktop.md) |
| Clipboard / Usb | **未实现** | [Clipboard](docs/portals/Clipboard.md) / [Usb](docs/portals/Usb.md) |

## ScreenCast 分享选择器（Omarchy）

采集仍由 Hyprland 负责；UI 为 `scripts/omarchy-share-picker` →
`SharePickerDialog.qml`，在 `xdph.conf` 中注册为 `custom_picker_binary`。

- **顶栏：** `Share region` · 多屏 Chip 过滤 · 搜索
- **内容：** Displays 网格 →「Windows」分隔 → Windows 网格  
  按宽度动态列数（约 ≥260px/卡，1–6 列）。显示器与窗口卡片样式一致。
- **缩略图：** 显示器 `grim -o`；窗口 `omarchy-portal-capture`（`hyprland_toplevel_export_v1`）
- **选择：** 默认选中第一项；↑↓←→ 移动；Enter / **Share** 确认；单击选中、双击确认
- **底栏（对齐 KDE）：** 左侧勾选 *Allow the application to do this without asking next time*；右侧 Cancel / Share
- **滚动条：** 右侧独立 gutter（不遮挡预览）；支持滚轮

细节见 [docs/portals/ScreenCast.md](docs/portals/ScreenCast.md)。

## 构建与安装

```bash
cargo build --release
./scripts/install-user.sh
```

用户安装会写入：

- `~/.local/libexec/xdg-desktop-portal-omarchy`
- `~/.local/libexec/omarchy-portal-capture`
- `~/.local/bin/omarchy-share-picker`
- `~/.local/share/xdg-desktop-portal/portals/omarchy.portal`
- `~/.config/xdg-desktop-portal/hyprland-portals.conf`
- `~/.config/systemd/user/xdg-desktop-portal-omarchy.service`
- `~/.config/omarchy/plugins/omarchy-portal/`（Quickshell UI）

需要时重启 portal / shell：

```bash
systemctl --user restart xdg-desktop-portal-omarchy xdg-desktop-portal xdg-desktop-portal-hyprland
omarchy restart shell
```

Omarchy 上 `XDG_CURRENT_DESKTOP=Hyprland`，实际生效的是 `hyprland-portals.conf`。

## Demo（不走 D-Bus）

```bash
cargo run -- --demo file-chooser
```

## 推荐路由配置

```ini
[preferred]
default=omarchy;hyprland;gtk
org.freedesktop.impl.portal.ScreenCast=hyprland
org.freedesktop.impl.portal.GlobalShortcuts=hyprland
org.freedesktop.impl.portal.InputCapture=hyprland
org.freedesktop.impl.portal.Screenshot=omarchy
org.freedesktop.impl.portal.Secret=gnome-keyring
```
