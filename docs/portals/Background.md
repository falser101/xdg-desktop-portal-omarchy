# Background

状态：**已实现**（对齐 KDE 三种结果 + 自启细节 + 状态信号）  
对照：`xdg-desktop-portal-kde` → `background.cpp`  
源码：`src/portals/background.rs`、`shell/omarchy.portal/BackgroundDialog.qml`

## 有什么用

沙箱 / Flatpak 应用要在**没有可见窗口时继续跑**，或写**登录自启**时，经 portal 询问用户。

## 已完成（对齐 KDE）

| 能力 | 行为 |
|------|------|
| `NotifyBackground` | 三结果：`0` Forbid / `1` Allow / `2` Allow once |
| 询问 UI | 对话框 **Deny / Allow once / Allow**（Omarchy toast 画不出按钮，不用通知形态） |
| 关掉未选 | **Allow once**（同 KDE 关掉通知） |
| 同 app 提示未关时再问 | 静默 **Allow once**（关窗后可再弹，同 KDE） |
| `EnableAutostart` | 写/删 `~/.config/autostart/<id>.desktop`；`DBusActivatable`（flags&1）；`X-Flatpak` |
| `GetAppState` | `hyprctl`：有窗=1，前台=2 |
| `RunningApplicationsChanged` | 监听 Hyprland `socket2`（open/close/active window） |

## 和 KDE 仍差的

| 项 | Omarchy | KDE |
|----|---------|-----|
| 询问载体 | 三按钮对话框 | 常驻通知 Allow/Deny |
| Deny 二次确认 | 无（直接 Forbid） | MessageBox「Deny Anyway」 |
| 应用 id | Hyprland `class` | PlasmaWindow + Flatpak info |
| 配置静默 Allow once | 无单独开关 | `NotifyBackgroundApps` |

## 怎么测试

```bash
python3 scripts/portal-call.py background --timeout 60000

# Deny → result 0 Forbid
# Allow → 1
# Allow once / 关窗 → 2
```

自启：

```bash
gdbus call --session \
  --dest org.freedesktop.impl.portal.desktop.omarchy \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.impl.portal.Background.EnableAutostart \
  'org.omarchy.portal.test' true "['/usr/bin/true']" 1
# 看 ~/.config/autostart/org.omarchy.portal.test.desktop（含 DBusActivatable、X-Flatpak）
```

改 QML 后：`./scripts/install-user.sh`，重启 portal；必要时 `omarchy restart shell`。
