# Inhibit

状态：**已实现**（idle / sleep + 锁屏监视）  
源码：`src/portals/inhibit.rs`

## 已完成

- `systemd-inhibit`：idle / sleep
- `CreateMonitor` + `StateChanged`：Hyprland socket2 的 lock / unlock → `screensaver-active`

## 延后

- 注销 / 切换用户 inhibit UI
- session-state 的 Query End / Ending（没有会话管理器对接）
- logind CanShutdown 协商
