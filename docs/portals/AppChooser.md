# AppChooser

状态：**已实现**（基础选应用可用；「设为默认」已接 mimeapps）  
对照：`xdg-desktop-portal-kde` → `appchooser.cpp` / `AppChooserDialog.qml`  
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

## 和 KDE 的差距

| 项 | Omarchy | KDE |
|----|---------|-----|
| 选应用 + 搜索 + 图标 | 有（列表） | 有（网格 + Kirigami） |
| `last_choice` / Default | 有 | 有；另用 `KApplicationTrader::preferredService` 标系统默认 |
| **设为默认（remember）** | **有** → `mimeapps.list` | 有 → `KApplicationTrader::setPreferredService` + `kbuildsycoca` |
| `content_type` 过滤推荐 | 依赖前端传入的 `choices`；空 choices 时列全部 | 可按 MIME `queryByMimeType`；「仅推荐 / 全部」切换 |
| **`UpdateChoices`** | **空实现** | 对话框打开后可热更新 preferred 列表 |
| **`activation_token`** | **未返回** | Wayland `KWaylandExtras::xdgActivationToken`，打开后焦点更稳 |
| `parent_window` + `modal` | 丢掉 | `Utils::setParentWindow` |
| Discover「找更多应用」 | 无 | `plasma-discover --mime …` |
| 终端 / 任意可执行路径 | 无 | 私有接口 + shellAccess（Plasma 集成） |
| URI → MIME（KIO） | 无（用前端给的 `content_type`） | `KIO::MimeTypeFinderJob`（私有 Open 路径） |
| 历史记录下拉 | 无 | `history` → ComboBox |

### 优先后续（按收益）

1. `activation_token`（Wayland 焦点）
2. 「仅推荐 / 显示全部」切换（按 `content_type` 筛 MimeType=）
3. `UpdateChoices` 接到打开中的对话框
4. `parent_window`（跨 portal 共用缺口）
5. Discover / 终端打开 —— 低优先级或不必对齐

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
