# xdg-desktop-portal-omarchy

[English](README.md)

Omarchy（Hyprland）的 [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/) 后端。对话框走 Omarchy Quickshell 插件。

## 已实现

| Portal | 能力 |
|--------|------|
| **FileChooser** | Open / Save / SaveFiles — 侧栏、过滤器、可折叠面包屑、搜索、预览、新建文件夹、Documents 路径还原 |
| **Settings** | 外观 / 强调色（沙箱应用） |
| **AppChooser** | 打开方式；可选设为默认 |
| **Account** | 用户名 / 头像 |
| **Access** | 权限确认（choices、图标） |
| **Screenshot** | 交互截图 + 取色 |
| **Background** | 允许 / 仅一次 / 禁止 |
| **DynamicLauncher** | 安装 / 卸载 Web 应用启动器 |
| **Notification** | 桥接到 Freedesktop 通知（action、图标、常驻） |
| **Inhibit** | 抑制空闲 / 休眠 |
| **Email** | 通过 `mailto:` / 邮件客户端起草（含附件） |
| **Wallpaper** | 设置壁纸 |
| **Lockdown** | 桩 |
| **ScreenCast UI** | 分享选择器 — Display / Windows / Region，直播 `ScreencopyView` 预览 |

ScreenCast / GlobalShortcuts / InputCapture 的**采集**仍由 `xdg-desktop-portal-hyprland` 负责；Secret 走 `gnome-keyring`。本仓库提供 Omarchy 风格的分享选择器（`omarchy-share-picker`）。

## 安装

```bash
yay -S xdg-desktop-portal-omarchy-git
xdg-desktop-portal-omarchy-setup
```

从源码：

```bash
./scripts/install-user.sh          # 用户目录（开发）
# 或
sudo ./scripts/install-system.sh   # 系统安装
xdg-desktop-portal-omarchy-setup
```

打包说明：[docs/packaging.md](docs/packaging.md)。

## 许可证

MIT
