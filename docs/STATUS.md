# 实现状态

记录 `xdg-desktop-portal-omarchy` **已经接上的接口**、**明确交给别的后端的接口**、以及 **还没做的接口**。日期以仓库当前 `main` 为准。

应用（Firefox、Chromium、Flatpak 等）只跟前端 `xdg-desktop-portal` 说话。本仓库实现的是后端 `org.freedesktop.impl.portal.desktop.omarchy`。Hyprland 上实际生效的路由文件是：

```
~/.config/xdg-desktop-portal/hyprland-portals.conf
```

对应源文件：`data/omarchy-portals.conf`。

## 架构

| 层 | 作用 |
|----|------|
| Rust daemon `xdg-desktop-portal-omarchy` | D-Bus 后端：导出 `org.freedesktop.impl.portal.*`，处理请求、抓取、写配置 |
| Quickshell 插件 `omarchy-portal` | 对话框 UI（Access / FileChooser / AppChooser / Screenshot / Share…） |
| `omarchy-share-picker` | 给 `xdg-desktop-portal-hyprland` 用的自定义分享选择器 |
| egui `--picker` 子进程 | shell 插件不可用时的后备对话框 |

对话框优先 `omarchy-shell shell summon omarchy-portal`。插件装在 `~/.config/omarchy/plugins/omarchy-portal/`（不改 `/usr/share/omarchy/`）。窗口是 Quickshell `FloatingWindow`（居中卡片），不是全屏 layer-shell。

```
应用
  → xdg-desktop-portal（前端）
    → omarchy（本仓库：FileChooser / Settings / Screenshot / …）
    → hyprland（ScreenCast 采集、GlobalShortcuts、InputCapture）
    → gnome-keyring（Secret）
    → gtk（上面都没有时的兜底）
```

---

## 已实现（本仓库后端）

### FileChooser — 文件选择

- Open / Save / SaveFiles
- Places：Home、XDG 用户目录（Downloads/Documents/Pictures/Videos/Music/Projects）、Computer=`/`，再叠加 `~/.config/gtk-3.0/bookmarks`
- 过滤器：portal glob + MIME（`image/*` 等会展开成扩展名）
- 新文件夹、隐藏文件、路径栏、搜索、覆写确认
- Save 时按当前过滤补扩展名；SaveFiles 在目录里生成不冲突文件名
- 预览：只在列表里选中**已存在**的图片或文本时显示

未做完：列表缩略图、按列排序、面包屑、最近文件、网络位置（KIO）、把 FileChooser `choices` 做成下拉而不只是开关、附着父窗口。

### Settings — 外观

- 读 `~/.local/state/omarchy/current/theme/colors.toml`
- `org.freedesktop.appearance`：`color-scheme`、`accent-color`、`contrast`、`reduced-motion`
- 主题文件变化时发 `SettingChanged`

未做完：字体、光标大小、完整 GNOME/KDE 设置命名空间。

### AppChooser — 用哪个应用打开

- 桌面文件列表、图标、搜索
- `last_choice` 标 Default
- 返回给前端的 id **不含** `.desktop`（前端会自己拼）

未做完：把选择写成系统默认 MIME 处理器。

### Account / Access

- Account：确认后返回用户名、GECOS 全名、`~/.face`
- Access：Allow / Deny，文案来自调用方

未做完：Access 的多选项 `choices` 在 QML 里还没做成完整控件（egui 后备有复选框）。

### Notification

- `omarchy-notification-send` / `omarchy-notification-dismiss`

未做完：通知按钮点回去的 `ActionInvoked`、图标字节、常驻通知。

### Inhibit

- `systemd-inhibit`：idle / sleep
- `CreateMonitor` + `StateChanged`：Hyprland socket2 的 lock / unlock → `screensaver-active`

未做完：注销/切换用户 inhibit UI；session-state 的 Query End / Ending（没有会话管理器对接）；KDE 那种 logind CanShutdown 协商。

### Email / Wallpaper / Lockdown

- Email：`xdg-email`
- Wallpaper：确认后 `omarchy-theme-bg-set`
- Lockdown：属性桩，默认允许本机桌面常见操作（location 默认 disable）

未做完：Wallpaper 预览图；Lockdown 跟系统策略联动。

### Screenshot

- 交互对话框：整屏 / 框选 / 当前窗口
- 非交互：直接 `grim`，存到 `~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png`
- 框选：`slurp`；当前窗口：`hyprctl activewindow`
- 取色：`hyprpicker -n -f hex`
- `AvailableTargets` = Screen | Area | Active Window（1|4|8）

未做完：任选一个窗口（target=2）；框选/窗口选择还是外部工具，不是 Omarchy 自绘 overlay。

### Background

- `NotifyBackground`：Allow（1）/ Forbid（0）
- `EnableAutostart`：写 `~/.config/autostart/<app-id>.desktop`
- `GetAppState`：按 `hyprctl clients` 尽力填（前台=2，有窗口=1）

未做完：Allow once（2）第三按钮；窗口开闭时发 `RunningApplicationsChanged`。

### DynamicLauncher

- `PrepareInstall`：确认安装，返回用户看到的 name（token 由前端生成）
- `RequestInstallToken`：只允许软件中心类 app id（GNOME Software / Discover / AppCenter）
- `SupportedLauncherTypes` = Application | Webapp

未做完：对话框里改名称/图标。

---

## 委托给其他后端（本仓库不实现协议，只路由）

这些在 `omarchy-portals.conf` 里写死，**不要**再在本 daemon 里重复实现采集引擎。

| 接口 | 后端 | 说明 |
|------|------|------|
| ScreenCast | hyprland | PipeWire / Hyprland 采集。**选择器**已换成 `~/.local/bin/omarchy-share-picker`（`xdph.conf` 的 `custom_picker_binary`），stdout 遵循 `[SELECTION]r?/screen:NAME\|window:ID\|region:OUT@x,y,w,h`，窗口 ID 来自 `XDPH_WINDOW_SHARING_LIST` |
| GlobalShortcuts | hyprland | Hyprland 协议 |
| InputCapture | hyprland | Hyprland 协议 |
| Secret | gnome-keyring | `org.freedesktop.impl.portal.Secret` → `org.freedesktop.secrets` |

分享选择器已有：显示器、窗口、区域（slurp）、restore token。未做完：窗口缩略图预览（hyprland-preview-share-picker 那种）。

---

## 未实现

下面这些本仓库 **没有** 导出，前端会落到 gtk / 没有后端 / 失败，取决于系统和调用方。

### 明确延后

| 接口 | 原因 |
|------|------|
| **Print** | 当初就定为 later。需要打印对话框 + CUPS/GTK print |
| RemoteDesktop | 远程桌面 / 输入注入，应继续跟 Hyprland/KWin 采集栈走 |
| Clipboard | 主要是 KDE 的 impl |
| Usb | 设备选择 UI + 权限 |

### 前端有、常见桌面很少自实现的 impl

Location、Camera、Trash、NetworkMonitor、ProxyResolver、MemoryMonitor、PowerProfile、GameMode、Realtime、OpenURI（前端用 AppChooser，没有单独 impl）等。没有计划除非有实际应用卡在上面。

---

## 对照表

| 接口 | 状态 | 谁来做 |
|------|------|--------|
| FileChooser | 已实现 | omarchy |
| Settings | 已实现 | omarchy |
| AppChooser | 已实现 | omarchy |
| Account | 已实现 | omarchy |
| Access | 已实现 | omarchy |
| Notification | 已实现 | omarchy |
| Inhibit | 已实现 | omarchy |
| Email | 已实现 | omarchy |
| Wallpaper | 已实现 | omarchy |
| Lockdown | 已实现（桩） | omarchy |
| Screenshot | 已实现 | omarchy |
| Background | 已实现 | omarchy |
| DynamicLauncher | 已实现 | omarchy |
| ScreenCast | 采集委托 + 选择器自研 | hyprland + `omarchy-share-picker` |
| GlobalShortcuts | 委托 | hyprland |
| InputCapture | 委托 | hyprland |
| Secret | 委托 | gnome-keyring |
| Print | **未实现** | — |
| RemoteDesktop | **未实现** | — |
| Clipboard | **未实现** | — |
| Usb | **未实现** | — |

---

## 自测

```bash
# 设置（无窗口）
python3 scripts/portal-call.py settings

# 文件 / 账号 / 打开链接（会弹出 Omarchy 对话框）
python3 scripts/portal-call.py open
python3 scripts/portal-call.py save
python3 scripts/portal-call.py open-dir
python3 scripts/portal-call.py account
python3 scripts/portal-call.py open-uri

# 截图 / 取色 / 后台（截图、取色会用 grim / hyprpicker）
python3 scripts/portal-call.py screenshot
python3 scripts/portal-call.py pick-color
python3 scripts/portal-call.py background
```

无 D-Bus 的 egui 后备：

```bash
cargo run -- --demo file-chooser
cargo run -- --demo access
```

屏幕分享用 OBS 或浏览器选窗口，走的是 Hyprland ScreenCast + `omarchy-share-picker`。

安装：`./scripts/install-user.sh`（用户级，不写 `/usr/share/omarchy/`）。插件改完若 keepLoaded 没热更新，执行 `omarchy restart shell`。
