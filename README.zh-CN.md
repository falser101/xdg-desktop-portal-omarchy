# xdg-desktop-portal-omarchy

[English](README.md)

Omarchy（Hyprland）的 [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) 后端。对话框用内置 egui。

## 已实现

| Portal | 能力 |
|--------|------|
| **FileChooser** | Open / Save / SaveFiles |
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

ScreenCast / GlobalShortcuts / InputCapture 由 `xdg-desktop-portal-hyprland` 负责，共享选择器是 [`omarchy-share-picker`](https://github.com/falser101/omarchy-share-picker)。Secret 走 `gnome-keyring`。

## 安装

```bash
yay -S xdg-desktop-portal-omarchy-git
xdg-desktop-portal-omarchy-setup
```

源码：`./scripts/install-user.sh`，或 `sudo ./scripts/install-system.sh` 后执行 `xdg-desktop-portal-omarchy-setup`。

打包：[docs/packaging.md](docs/packaging.md)。

## 许可证

MIT
