# DynamicLauncher

状态：**已实现**  
对照：KDE DynamicLauncherDialog  
源码：`src/portals/dynamic_launcher.rs`

## 已完成

- `PrepareInstall`：确认安装，返回用户看到的 name（token 由前端生成）
- `RequestInstallToken`：只允许软件中心类 app id（GNOME Software / Discover / AppCenter）
- `SupportedLauncherTypes` = Application | Webapp

## 延后

- 对话框里改名称 / 图标
