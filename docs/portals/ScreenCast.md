# ScreenCast

状态：**委托采集 + 自研选择器**  
对照：KDE ScreenCast（KWin / PipeWire）；本机采集走 Hyprland  
源码：选择器 `scripts/omarchy-share-picker`；路由在 `omarchy-portals.conf` / `xdph.conf`

## 已完成

- 采集委托 `xdg-desktop-portal-hyprland`
- 自定义分享选择器：显示器、窗口、区域（slurp）、restore token
- stdout 遵循 `[SELECTION]r?/screen:NAME|window:ID|region:OUT@x,y,w,h`

## 延后

- 窗口缩略图预览（`hyprland-preview-share-picker` 那种）
- 不在本 daemon 里重复实现 PipeWire 采集引擎
