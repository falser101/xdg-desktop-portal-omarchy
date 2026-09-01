# Settings

状态：**已实现**（标准 `org.freedesktop.appearance` 够用）  
源码：`src/portals/settings.rs`、`src/theme.rs`

## 有什么用

Settings **不是**「系统设置 App」，也**不能写**配置。它是给应用 / 工具箱的**只读外观探针**，类似旧式 XSettings：

- Flatpak / Snap 沙箱里读不到主机桌面设置
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


## 延后

按需求优先级：

1. 把 `reduced-motion` / `contrast` 接到真实系统开关（若 Omarchy 有对应设置）
2. 字体名、光标大小（若沙箱 GTK 应用抱怨不一致）
3. 更完整的 `org.gnome.desktop.interface`（icon-theme 别写死等）
4. **不计划** Plasma 专用 Settings 命名空间（除非有具体应用卡住）

## 自测

```bash
python3 scripts/portal-call.py settings

busctl --user call org.freedesktop.portal.Desktop \
  /org/freedesktop/portal/desktop org.freedesktop.portal.Settings \
  ReadAll 'as' 1 'org.freedesktop.appearance'
```

换主题后再读一次，或监听 `SettingChanged`，确认暗色/强调色会更新。
