# AppChooser

状态：**已实现**（基础选应用可用；「设为默认」已接 mimeapps）  
源码：`src/portals/app_chooser.rs`、`shell/omarchy.portal/AppChooserDialog.qml`、`src/desktop.rs`、`src/ui/app_chooser.rs`（egui 后备）

## 有什么用

应用（或前端 OpenURI）问：「用哪个桌面应用打开这个 URI / 文件？」用户选一个，后端返回 `choice`（**不含** `.desktop` 后缀，前端自己拼）。

常见触发：

- Flatpak / 浏览器 `OpenURI` 且 `ask=true`（「总是询问」）
- 桌面「打开方式」类流程（经 portal）

## 已完成

- 桌面文件列表、图标、搜索、双击打开
- `choices` 非空时只列这些；为空时列全部已安装应用
- `last_choice` 标 Default 并预选
- 副标题显示 `uri` / `filename` / `content_type`
- 返回 id **不含** `.desktop`
- UI 选中态对齐 launcher（`Color.menu`）
- **「设为默认打开方式」**：勾选后写入 `~/.config/mimeapps.list` 的 `[Default Applications]`（并追加 `[Added Associations]`）
- **图标解析**：把 `Icon=` 解析成 hicolor / Papirus / pixmaps 的绝对路径（不依赖当前 `breeze-dark`）；URL handler 无 Icon 时尝试继承主应用；加载失败再回退通用图标


## 怎么测试

### 1. 日常：OpenURI 询问（走完整前端）

```bash
# 会弹 AppChooser（ask=true）
python3 scripts/portal-call.py open-uri
```

选一个浏览器后应打开 `https://omarchy.org`。若勾了「设为默认」，再查：

```bash
grep -E 'x-scheme-handler/https|text/html' ~/.config/mimeapps.list
```

### 2. 指定 URI / MIME（脚本）

```bash
python3 scripts/portal-call.py app-chooser
python3 scripts/portal-call.py app-chooser --uri 'https://example.com'
python3 scripts/portal-call.py app-chooser --uri 'file:///etc/hosts' --mime text/plain
```

### 3. 无 D-Bus 的 egui 后备

```bash
cargo run -- --demo app-chooser
```

### 4. 真机

- 在会走 portal 的应用里打开链接并选「总是询问」
- 确认路由：`~/.config/xdg-desktop-portal/hyprland-portals.conf` 里 `AppChooser=omarchy`
- 插件改完：`./scripts/install-user.sh` 或拷贝 QML 后 `omarchy restart shell`，并重启 portal 服务

```bash
systemctl --user restart xdg-desktop-portal-omarchy.service xdg-desktop-portal.service
```

### 5. 确认打到 omarchy

弹窗时日志应有 `AppChooser.ChooseApplication` / `AppChooser selected`：

```bash
journalctl --user -u xdg-desktop-portal-omarchy.service -f
```
