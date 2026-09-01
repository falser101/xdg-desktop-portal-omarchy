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
| **顶栏** | 左侧 `Share region`；多屏时 Chip 按几何过滤；右侧搜索 |
| **内容** | Displays 网格 →「Windows」分隔 → Windows 网格（单页滚动） |
| **列数** | 按对话框宽度动态增加（约 ≥260px/卡，1–6 列）；键盘网格同步 |
| **卡片** | Displays 与 Windows **同一套**卡片样式与单元格宽度（图标+标题头栏 + 预览）；无双层预览背景 |
| **滚动条** | 右侧独立 gutter，不遮挡预览；可拖 + 滚轮 |
| **底栏** | 左侧勾选；右侧 Cancel / Share（与勾选同一行） |

### 交互

- 默认选中第一项
- ↑↓←→ 在 Displays / Windows 网格间移动选中
- Enter / **Share** 确认；单击选中、双击确认；Esc 取消
- 底栏勾选：  
  **Allow the application to do this without asking next time**（restore token）
- `Share region`：走 `omarchy-capture-region smart`（冻屏 + 吸附）；整屏吸附发 `screen:NAME`；自由框选发 `region:OUT@x,y,w,h`

### 缩略图与数据

- 显示器：`grim -o`
- 窗口：`omarchy-portal-capture` → `hyprland_toplevel_export_v1`（重叠窗口不串台）
- 过滤标题为 `Omarchy Portal` 的自身窗口
- stdout：`[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`
- 窗口 ID 来自 `XDPH_WINDOW_SHARING_LIST`；缩略图用 `hyprctl` address

## 预览（先弹窗再出画）

- 对话框立刻打开；卡片用 Quickshell `ScreencopyView`（`live` 在悬停/选中时开启）
- 显示器 → `Quickshell.screens`；窗口 → `Hyprland.toplevels` 按 address 解析
- 实时预览用 Quickshell `ScreencopyView`（Hyprland 合成器采集）
- 不再用 grim / `omarchy-portal-capture` 预抓 PNG（曾导致 OBS 超时、堵死 xdph）

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
