# Email

状态：**已实现**  
源码：`src/portals/email.rs`

## 已完成

- `ComposeEmail` → `xdg-email`（异步 `tokio::process`，不阻塞 runtime）
- 字段：`address` / `addresses` / `cc` / `bcc` / `subject` / `body` / `attachments`
- `activation_token` → 子进程 `XDG_ACTIVATION_TOKEN` + `DESKTOP_STARTUP_ID`
- 附件：接受 `file://` 或绝对本地路径；其它跳过并 `warn`
- 测试：`python3 scripts/portal-call.py email [--attach PATH] [--cc ADDR] …`

## 说明

- **无 shell 对话框**：直接拉起默认 `mailto:` 客户端。
- Frontend `attachment_fds` 由 xdg-desktop-portal 转成 impl 的 `attachments` URI。
- 附件 / CC 是否生效取决于本机默认邮件客户端（web mailto 往往不支持附件）。

## 延后

- 无；若某客户端对字段不完整再针对性补
