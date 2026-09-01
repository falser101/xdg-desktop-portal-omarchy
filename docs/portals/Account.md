# Account

状态：**已实现**（QML 对齐 KDE UserInfoDialog）  
对照：`xdg-desktop-portal-kde` → `account.cpp` / `UserInfoDialog.qml`  
源码：`src/portals/account.rs`、`shell/omarchy.portal/AccountDialog.qml`、`src/ui/confirm.rs`（egui 后备）

## 有什么用

应用通过 `org.freedesktop.portal.Account.GetUserInformation` 请求当前用户的**用户名、全名、头像**。用户确认（Share）后才返回；取消则失败。

日常比 FileChooser / ScreenCast 少见：Flatpak / 沙箱应用要展示「当前用户」资料时才会弹。

## 已完成（对齐 KDE）

- 窗口标题：`User Information Requested`
- 主文案：`Share user info with {应用名}?`（无 desktop 时回退 app_id / `this application`）
- 说明：会看到 username / full name / profile picture；带 `reason` 或「未提供原因」
- 大头像居中 + 全名 + 用户名（弱化色）
- 按钮：`Share` / `Cancel`
- 数据：优先 AccountsService（`UserName` / `RealName` / `IconFile`），否则 passwd GECOS + `~/.face`
- 无头像时回退主题图标 `avatar-default` / `user-identity` / `user`；返回 URI 时空则 `file://`（同 KDE）

## 和 KDE 仍差的

| 项 | Omarchy | KDE |
|----|---------|-----|
| 布局 / 文案 / Share | 有 | 有 |
| AccountsService + 头像回退 | 有 | 有 |
| `parent_window` | 仍丢掉 | 有 |
| UI 框架 | Omarchy 卡片 | Kirigami Avatar |

`parent_window` 与其它对话框共用，后面一起做。

## 怎么测试

```bash
# 走完整 portal（会弹 QML 对话框）
python3 scripts/portal-call.py account

# 期望：标题含应用名或 “this application”、大头像、全名/用户名、Share
# 点 Share → response 0，results 含 id / name / image(file://…)
# 点 Cancel → 取消

# egui 后备（不经 shell）
cargo run -- --demo account
```

改完 QML 后：`./scripts/install-user.sh`（或至少拷贝插件），`omarchy restart shell`，并重启 portal 服务。
