# Notification

状态：**已实现**（薄桥 → Omarchy FDO / shell，含 action / 图标 / 常驻）  
源码：`src/portals/notification.rs`

## 和本机通知的关系

Omarchy **真正的通知服务**在 shell：

```
本机 App / omarchy-notification-send
  → org.freedesktop.Notifications.Notify
    → quickshell（plugins/notifications）
```

本 portal 给沙箱 / Flatpak 当桥，并直接调 FDO `Notify`（不再只包一层 CLI）：

```
沙箱 App
  → xdg-desktop-portal
    → omarchy AddNotification / RemoveNotification
      → org.freedesktop.Notifications.Notify / CloseNotification
        → shell 弹窗
      ← FDO ActionInvoked
        → portal ActionInvoked 或 org.freedesktop.Application.ActivateAction
```

日常本机脚本 **不经过**本接口。

## 关于 KDE

KDE 已从 `xdg-desktop-portal-kde` **删除** Notification，改由 plasmashell 直接实现。  
Omarchy 仍用「portal 守护进程薄桥 + shell FDO」，不必做成第二套通知 UI。

## 已完成

| 能力 | 行为 |
|------|------|
| title / body / markup-body / priority | → FDO summary/body + urgency |
| 图标名（字符串） | → `app_icon` |
| 图标 `themed` | 取名称列表第一项 |
| 图标 `bytes` / `file-descriptor` | 写入临时文件 → `image-path` hint |
| `default-action` (+ target) | FDO action id `default`（点整条 toast） |
| `buttons` | FDO actions；点按钮触发 |
| `display-hint` 含 `persistent` | `resident` / `persistence` hints，`expire_timeout=0` |
| `RemoveNotification` | `CloseNotification`（按 FDO id） |
| Action 回传 | `app.*` → `ActivateAction`；其它 → `Activate` + 发 impl `ActionInvoked` |

## 怎么测试

```bash
# 发一条带按钮 + 常驻的 portal 通知
python3 scripts/portal-call.py notification

# 点 toast 或 Open；journal 应出现 action invoked
journalctl --user -u xdg-desktop-portal-omarchy -f

# 关掉
python3 scripts/portal-call.py notification-remove

# 或 gdbus
gdbus call --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Notification.AddNotification \
  'test-1' \
  "{
    'title': <'按钮测试'>,
    'body': <'点 Open'>,
    'icon': <'dialog-information'>,
    'default-action': <'open-main'>,
    'buttons': <[{'label': <'Open'>, 'action': <'open-main'>}]>,
    'display-hint': <['persistent']>
  }"
```

确认路由：`Notification=omarchy`（`hyprland-portals.conf`）。改完需 `./scripts/install-user.sh` 并 `systemctl --user restart xdg-desktop-portal-omarchy`。
