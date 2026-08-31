# Access

状态：**已实现**（含 QML choices / icon，对齐 KDE 主路径）  
对照：`xdg-desktop-portal-kde` → `access.cpp` / `AccessDialog.qml`  
源码：`src/portals/access.rs`、`shell/omarchy.portal/AccessDialog.qml`、`src/ui/confirm.rs`（egui 后备）

## 有什么用

通用权限确认框：调用方传入 `title` / `subtitle` / `body`，用户 Allow / Deny。可选 `choices`（开关或下拉）、`icon`、按钮文案。

日常比 FileChooser 少见；Flatpak / 库要一次性确认，或其它流程复用 Access 时才会弹。

## 已完成

- Allow / Deny，文案与 `deny_label` / `grant_label`
- **`choices`**：空 options → 开关；有 options → 下拉；Allow 时返回选中值（QML + egui）
- **`icon`**：读 options，默认 `dialog-question`；解析失败回退问号图标
- egui 后备：checkbox + ComboBox

## 和 KDE 仍差的

| 项 | Omarchy | KDE |
|----|---------|-----|
| choices / icon / 按钮文案 | 有 | 有 |
| `parent_window` + `modal` | 仍丢掉 | 有 |
| UI 框架 | Omarchy 卡片 | Kirigami FormLayout |

`parent_window` 与其它对话框共用，后面一起做。

## 怎么测试

```bash
# 带 choices + icon（直调 omarchy 后端，会弹窗）
python3 scripts/portal-call.py access

# 期望：图标、开关「Remember this decision」、下拉「Access scope」
# 点 Allow 后 reply 里 choices 应含 remember / scope 的当前值

# egui 后备
cargo run -- --demo access
```

改完 QML 后：`./scripts/install-user.sh` 或拷贝插件，`omarchy restart shell`，并确保 portal 服务在跑。
