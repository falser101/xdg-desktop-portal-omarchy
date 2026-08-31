# Background

状态：**已实现**  
对照：KDE Background  
源码：`src/portals/background.rs`

## 已完成

- `NotifyBackground`：Allow（1）/ Forbid（0）
- `EnableAutostart`：写 `~/.config/autostart/<app-id>.desktop`
- `GetAppState`：按 `hyprctl clients` 尽力填（前台=2，有窗口=1）

## 延后

- Allow once（2）第三按钮
- 窗口开闭时发 `RunningApplicationsChanged`
