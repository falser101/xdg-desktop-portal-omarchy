# xdg-desktop-portal-omarchy

[English](README.md)

Omarchy（Hyprland）的 [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) 后端。对话框用内置 egui。

Omarchy 设置 `XDG_CURRENT_DESKTOP=Omarchy:Hyprland`。xdg-desktop-portal 会加载 `/usr/share/xdg-desktop-portal/omarchy-portals.conf`，并通过 D-Bus 激活本后端。不需要每用户 setup。

## 已实现

| Portal | 能力 |
|--------|------|
| **FileChooser** | Open / Save / SaveFiles，图片缩略图 |
| **Settings** | 外观 / 强调色 |
| **AppChooser** | 打开方式；可选设为默认 |
| **Account** | 用户名 / 头像 |
| **Access** | 权限确认 |
| **Screenshot** | 交互截图 + 取色 |
| **Background** | 允许 / 仅一次 / 禁止 |
| **DynamicLauncher** | 安装 / 卸载 Web 应用启动器 |
| **Notification** | Freedesktop 通知桥 |
| **Inhibit** | 抑制空闲 / 休眠 |
| **Email** | `mailto:` 起草（含附件） |
| **Wallpaper** | 设置壁纸 |
| **Lockdown** | 桩（默认关闭定位） |

ScreenCast / GlobalShortcuts / InputCapture 由 `xdg-desktop-portal-hyprland` 负责。Print 走 `xdg-desktop-portal-gtk`。Secret 走 `gnome-keyring`。

## 安装

随 Omarchy 打包，或：

```bash
yay -S xdg-desktop-portal-omarchy-git
```

源码：`./scripts/install-user.sh`，或 `sudo ./scripts/install-system.sh`。重新加载 Hyprland 或重新登录，使 `XDG_CURRENT_DESKTOP` 包含 Omarchy。

打包：[docs/packaging.md](docs/packaging.md)。

## 许可证

MIT
