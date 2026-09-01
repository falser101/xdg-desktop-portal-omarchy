# ScreenCast

状态：**委托采集 + Omarchy 预览选择器**  
对照：KDE ScreenCast（KWin live PipeWire 预览）；本机采集走 Hyprland  
源码：`scripts/omarchy-share-picker`、`shell/omarchy.portal/SharePickerDialog.qml`；路由在 `omarchy-portals.conf` / `xdph.conf`

## 已完成

- 采集委托 `xdg-desktop-portal-hyprland`（PipeWire / session / restore）
- 自定义分享选择器（`custom_picker_binary`）：
  - **Displays / Windows** 卡片网格
  - **显示器缩略图**：`grim -o`
  - **窗口缩略图**：`omarchy-portal-capture` → `hyprland_toplevel_export_v1`（与 hyprland-preview-share-picker 同协议；按窗口 surface 导出，重叠窗口不串台）
  - 窗口搜索；过滤 Omarchy Portal 自身窗口
  - 区域：`omarchy-capture-region smart`（冻屏 + 吸附窗口/显示器，与 Print Screen 同套）
  - 整屏吸附时发 `screen:NAME`；自由框选发 `region:OUT@x,y,w,h`（相对输出）
  - restore token 开关（随 reply 回传）
- UI：`Color.popups` / `Color.menu` / `Style.font`，跟 Omarchy 主题
- stdout 遵循 `[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`
- 窗口 ID 来自 `XDPH_WINDOW_SHARING_LIST`（分享用）；缩略图用 `hyprctl` address 走 toplevel-export

## 延后

- live PipeWire / Quickshell `ScreencopyView` 预览
- 定时刷新静态缩略图（半 live）
- 自绘区域 overlay（现用官方 `omarchy-capture-region`）
- 不在本 daemon 里重复实现 PipeWire 采集引擎

## 自测

```bash
# 确认 xdph 指向 Omarchy 选择器
grep custom_picker_binary ~/.config/hypr/xdph.conf

# 用 OBS / 浏览器「共享屏幕」应弹出 Omarchy 卡片预览
# 或重装后：
./scripts/install-user.sh
```
