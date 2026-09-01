# Screenshot

状态：**已实现**  
源码：`src/portals/screenshot.rs`、`shell/omarchy.portal/ScreenshotDialog.qml`

## 已完成

- 交互对话框：整屏 / 框选 / 当前窗口
- 非交互：`grim` → `~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png`
- 框选：`slurp`；当前窗口：`hyprctl activewindow`
- 取色：`hyprpicker -n -f hex`
- `AvailableTargets` = Screen | Area | Active Window（1|4|8）
- UI 选中态对齐 launcher

## 延后

- 任选一个窗口（target=2）
- 框选 / 窗口选择改为 Omarchy 自绘 overlay（现在仍用外部工具）
