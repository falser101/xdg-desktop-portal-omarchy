# Settings

状态：**已实现**（标准 `org.freedesktop.appearance` 够用）  
对照：`xdg-desktop-portal-kde` → `src/settings.cpp`  
源码：`src/portals/settings.rs`、`src/theme.rs`

## 有什么用

Settings **不是**「系统设置 App」，也**不能写**配置。它是给应用 / 工具箱的**只读外观探针**，类似旧式 XSettings：

- Flatpak / Snap 沙箱里读不到主机 `gsettings` / `kdeglobals`
- 通过 portal 问「系统现在偏暗色吗？强调色是什么？」
- 改主题时后端发 `SettingChanged`，应用可以**热切换**暗/亮，不用重启

官方文档也写了：只暴露一小撮主机设置，**不是**通用配置存储。

## 举例

| 场景 | 读什么 | 效果 |
|------|--------|------|
| Chromium / Chrome / Edge | `org.freedesktop.appearance` → `color-scheme`、`accent-color` | 跟系统暗色模式；强调色（有 feature flag） |
| Firefox（Flatpak 或走 portal 的构建） | 同上 | `about:preferences` 里「跟随系统主题」 |
| Electron / VS Code / Slack 等 | `color-scheme` | 界面亮/暗跟 Omarchy 主题 |
| GTK4 / libadwaita 应用 | `org.freedesktop.appearance` + 可选 `org.gnome.desktop.interface` | Adwaita 暗色 / 强调色 |
| Qt 应用（在非 Plasma 上） | 通常走 `org.freedesktop.appearance` | 暗色偏好 |
| 自测 | `python3 scripts/portal-call.py settings` | 打印当前值 |

本机当前大致行为（Omarchy 主题驱动）：

- `color-scheme = 1` → prefer dark；`2` → prefer light；`0` → no preference
- `accent-color = (r,g,b)`，每个分量 0.0–1.0（来自 `colors.toml`）
- 换 `omarchy` 主题后应发 `SettingChanged`（监视 `~/.local/state/omarchy/current`）

## 已完成（Omarchy）

- 读 `~/.local/state/omarchy/current/theme/colors.toml`
- **`org.freedesktop.appearance`**
  - `color-scheme`（u）
  - `accent-color`（(ddd)）
  - `contrast`（固定 `0`）
  - `reduced-motion`（固定 `0`）
- **`org.gnome.desktop.interface`**（兼容子集，给 GTK 系）
  - `color-scheme`（`prefer-dark` / `prefer-light`）
  - `gtk-theme`（`Adwaita` / `Adwaita-dark`）
  - `icon-theme`（目前写死 `Yaru-blue`）
  - `text-scaling-factor`（固定 `1.0`）
- 主题目录变化 → `SettingChanged`（color-scheme / accent-color）

## 和 KDE 的差距

| 项 | Omarchy | KDE |
|----|---------|-----|
| 标准外观 | `appearance` 四键都有；contrast / reduced-motion **写死 0** | 有；`reduced-motion` 跟 `kdeglobals` 的 `AnimationDurationFactor==0`；contrast 亦未必完整 |
| 数据源 | Omarchy `colors.toml` | QPalette / `kdeglobals` / KWin |
| GNOME 兼容命名空间 | 只暴露 4 个键 | 一般不靠这套；GTK 侧常另有 `xdg-desktop-portal-gtk` 读完整 gsettings |
| **`org.kde.kdeglobals.*`** | **无** | 几乎整份 kdeglobals（字体、图标主题、widgetStyle、ColorScheme…）给 plasma-integration / Qt |
| **`org.kde.VirtualKeyboard`** | **无** | 跟 KWin 虚拟键盘状态 |
| **`org.kde.TabletMode`** | **无** | 跟 KWin 平板模式 |
| 字体 / 光标大小 | **无**（延后） | 经 kdeglobals（如 `font`）+ 变更信号 |
| 热更新 | 监视主题文件，发 appearance 信号 | palette / kdeglobals / KWin 属性变更都发 |

### 哪些差距重要

1. **日常跨平台应用（浏览器、Flatpak GTK/Electron）**  
   只认 `org.freedesktop.appearance`。Omarchy 与 KDE **差距不大**，暗色 + 强调色已经覆盖主路径。

2. **contrast / reduced-motion**  
   规范有键；Omarchy 恒为「无偏好」。无障碍用户、无障碍动效开关时会不对。KDE 至少接了 reduced-motion。

3. **`org.kde.*` 命名空间**  
   只对「以为自己在 Plasma 上」的 Qt/KDE 应用有用。Omarchy 上本来就不是 Plasma，**通常不必追平**。

4. **字体 / 光标 / 完整 GNOME interface**  
   若希望沙箱 GTK 应用字号、光标主题也跟主机一致，需要补；否则应用用自己的默认或 gtk 后端兜底。

## 延后

按需求优先级：

1. 把 `reduced-motion` / `contrast` 接到真实系统开关（若 Omarchy 有对应设置）
2. 字体名、光标大小（若沙箱 GTK 应用抱怨不一致）
3. 更完整的 `org.gnome.desktop.interface`（icon-theme 别写死等）
4. **不计划** 完整 `org.kde.kdeglobals.*` / VirtualKeyboard / TabletMode（除非有具体应用卡住）

## 自测

```bash
python3 scripts/portal-call.py settings

busctl --user call org.freedesktop.portal.Desktop \
  /org/freedesktop/portal/desktop org.freedesktop.portal.Settings \
  ReadAll 'as' 1 'org.freedesktop.appearance'
```

换主题后再读一次，或监听 `SettingChanged`，确认暗色/强调色会更新。
