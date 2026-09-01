# ScreenCast

[English](ScreenCast.en.md)

状态：**委托采集 + Omarchy 预览选择器**  
源码：`scripts/omarchy-share-picker`、`shell/omarchy.portal/SharePickerDialog.qml`、`src/bin/omarchy_portal_capture.rs`  
路由：`omarchy-portals.conf` / `~/.config/hypr/xdph.conf`（`custom_picker_binary`）

## 已完成

- 采集委托 `xdg-desktop-portal-hyprland`（PipeWire / session / restore）
- 自定义分享选择器（`custom_picker_binary` = `omarchy-share-picker`），Omarchy 布局：

### UI 布局

| 区域 | 内容 |
|------|------|
| **顶栏** | **Display / Windows / Region** 分页；Windows 多屏时 Chip 按几何过滤；右侧搜索 |
| **Display** | 整屏卡片：`Quickshell.screens` 作为 `ScreencopyView` 源（与 window-preview 相同）；名称 + 分辨率 |
| **Windows** | 窗口网格（图标 + 标题 + toplevel 预览） |
| **Region** | 独立页：说明 + `Select region`（`omarchy-capture-region`） |
| **滚动条** | 右侧独立 gutter，不遮挡预览；可拖 + 滚轮 |
| **底栏** | 左侧勾选；右侧 Cancel / Share（与勾选同一行） |

### 交互

- 默认 Display 页、选中第一台显示器
- ↑↓←→ 在当前页的卡片间移动；Enter / **Share** 确认；单击选中、双击确认；Esc 取消
- 底栏勾选：  
  **Allow the application to do this without asking next time**（restore token）
- Region：走 `omarchy-capture-region smart`（冻屏 + 吸附）；整屏吸附发 `screen:NAME`；自由框选发 `region:OUT@x,y,w,h`

### 缩略图与数据

- 显示器：Quickshell `ScreencopyView` + `ShellScreen`（`Quickshell.screens`，与 Omarchy window-preview 同一条路径）
- 窗口：`ScreencopyView` + `Hyprland.toplevels`（`hyprland_toplevel_export_v1`）
- 过滤标题为 `Omarchy Portal` 的自身窗口
- stdout：`[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`
- 窗口 ID 来自 `XDPH_WINDOW_SHARING_LIST`；缩略图用 `hyprctl` address

## 预览（先弹窗再出画）

- 对话框立刻打开；卡片用 Quickshell `ScreencopyView`（仍帧 + `captureFrame`，与 window-preview 一致）
- 显示器直接把 `Quickshell.screens` 的 `ShellScreen` 交给 `captureSource`，不做名称二次查找、不预抓 PNG
- 不再用 grim 预抓（曾导致 OBS 超时、堵死 xdph）

## 延后

- Virtual screen / Workspace 合成输出
- 自绘区域 overlay（现用官方 `omarchy-capture-region`）
- 不在本 daemon 里重复实现 PipeWire 采集引擎
- 可见区限流 / 减轻整屏「镜子效应」

## 自测

```bash
# 确认 xdph 指向 Omarchy 选择器
grep custom_picker_binary ~/.config/hypr/xdph.conf

# 重装并重启
./scripts/install-user.sh
systemctl --user restart xdg-desktop-portal-omarchy xdg-desktop-portal xdg-desktop-portal-hyprland
omarchy restart shell

# 用 OBS / 浏览器「共享屏幕」应弹出 Omarchy 卡片预览
```
