# ScreenCast

[English](ScreenCast.en.md)

状态：**委托采集 + Omarchy 预览选择器**  
对照：KDE `ScreenChooserDialog`（KWin live PipeWire 预览）；本机采集走 Hyprland  
源码：`scripts/omarchy-share-picker`、`shell/omarchy.portal/SharePickerDialog.qml`、`src/bin/omarchy_portal_capture.rs`  
路由：`omarchy-portals.conf` / `~/.config/hypr/xdph.conf`（`custom_picker_binary`）

## 已完成

- 采集委托 `xdg-desktop-portal-hyprland`（PipeWire / session / restore）
- 自定义分享选择器（`custom_picker_binary` = `omarchy-share-picker`），布局对齐 KDE：

### UI 布局

| 区域 | 内容 |
|------|------|
| **顶栏** | 左侧 `Share region`；多屏时 Chip 按几何过滤；右侧搜索 |
| **内容** | Displays 网格 →「Windows」分隔 → Windows 网格（单页滚动） |
| **列数** | 按对话框宽度动态增加（约 ≥260px/卡，1–6 列）；键盘网格同步 |
| **卡片** | Displays 与 Windows **同一套**卡片样式与单元格宽度（图标+标题头栏 + 预览）；无双层预览背景 |
| **滚动条** | 右侧独立 gutter，不遮挡预览；可拖 + 滚轮 |
| **底栏** | 左侧勾选（KDE 文案）；右侧 Cancel / Share（与勾选同一行） |

### 交互

- 默认选中第一项
- ↑↓←→ 在 Displays / Windows 网格间移动选中
- Enter / **Share** 确认；单击选中、双击确认；Esc 取消
- 底栏勾选文案对齐 KDE：  
  **Allow the application to do this without asking next time**（restore token）
- `Share region`：走 `omarchy-capture-region smart`（冻屏 + 吸附）；整屏吸附发 `screen:NAME`；自由框选发 `region:OUT@x,y,w,h`

### 缩略图与数据

- 显示器：`grim -o`
- 窗口：`omarchy-portal-capture` → `hyprland_toplevel_export_v1`（重叠窗口不串台）
- 过滤标题为 `Omarchy Portal` 的自身窗口
- stdout：`[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`
- 窗口 ID 来自 `XDPH_WINDOW_SHARING_LIST`；缩略图用 `hyprctl` address

## 延后

- live PipeWire / Quickshell `ScreencopyView` 预览（KDE 用 `PipeWireSourceItem`）
- 定时刷新静态缩略图（半 live）
- Virtual screen / Workspace 合成输出（KDE `OutputsModel` 可选项）
- 自绘区域 overlay（现用官方 `omarchy-capture-region`）
- 不在本 daemon 里重复实现 PipeWire 采集引擎

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
