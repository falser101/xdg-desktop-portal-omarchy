# DynamicLauncher

状态：**已实现**  
对照：KDE DynamicLauncherDialog  
源码：`src/portals/dynamic_launcher.rs`  
UI：`shell/omarchy.portal/DynamicLauncherDialog.qml`

## 已完成

- `PrepareInstall`：KDE 风格对话框（大图标、名称、可选 Webapp URL）
- 返回 `name` + 原样回传 `icon`（token 由前端生成；缺 `icon` 前端会把成功改成 response=2）
- `Edit Info…`：可改名称（`editable_name`，默认 true）
- `RequestInstallToken`：只允许软件中心类 app id（GNOME Software / Discover / AppCenter）
- `SupportedLauncherTypes` = Application | Webapp

## 自测

```bash
# 确认对话框（Application）——点 Create / Cancel；可点 Edit Info… 改名
python3 scripts/portal-call.py dynamic-launcher --timeout 60000

# Webapp 变体（显示目标 URL）
python3 scripts/portal-call.py dynamic-launcher-webapp --uri https://omarchy.org --timeout 60000

# 确认后立刻 Install 一个测试 .desktop
python3 scripts/portal-call.py dynamic-launcher --install --timeout 60000

# 清掉测试入口
python3 scripts/portal-call.py dynamic-launcher-uninstall

# 白名单免确认（无对话框）
python3 scripts/portal-call.py dynamic-launcher-token --app-id org.kde.discover          # response 0
python3 scripts/portal-call.py dynamic-launcher-token --app-id org.omarchy.portal.test    # response 2
```

期望：窗口标题 `Launcher Requested`；正文含大图标与名称；点 **Create** 时 `response: 0` 且 results 含 `token`。

## 延后

- Edit 时更换图标（KDE `IconDialog`；portal 默认 `editable_icon=false`）
