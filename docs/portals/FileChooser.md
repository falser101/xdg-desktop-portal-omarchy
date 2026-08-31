# FileChooser

状态：**已实现**（日常本机 Open/Save 可用）  
对照：`xdg-desktop-portal-kde` → `KFileWidget`  
源码：`src/portals/file_chooser.rs`、`shell/omarchy.portal/FileChooserDialog.qml`、`src/ui/file_chooser.rs`（egui 后备）

## 已完成

- Open / Save / SaveFiles
- Places：Recent（`recently-used.xbel`）、Home、XDG 用户目录、Computer=`/`、GTK bookmarks；Home 别名去重
- 过滤器：portal glob + MIME 展开；结果带 `current_filter`
- 新文件夹、隐藏文件、面包屑（末段可编辑，Ctrl+L 编完整路径）、搜索、覆写确认
- 列表按 Name / Size / Modified 排序；图片行缩略图 / 其它 MIME 字形
- Save：按当前过滤补扩展名；`current_file` 只给 basename 时预填；SaveFiles 生成不冲突文件名
- `choices`：开关或下拉；Open 结果带 `writable=true`
- 预览：选中已存在的图片或文本时显示侧栏
- UI：`Color.popups` / `Color.menu` 选中态，与 launcher 一致

## 延后（未对齐 KDE，后面再做）

按投入/收益排序：

1. **`parent_window` + `modal`**  
   KDE 用 `Utils::setParentWindow` 挂到调用方窗口。Omarchy 当前丢掉 `parent_window`，对话框是独立 `FloatingWindow`，无窗口级模态。

2. **沙箱路径还原（Documents + KIOFuse）**  
   Flatpak 传入 `/run/user/.../doc/...` 时，KDE 会还原成用户可认路径。Omarchy 无此层。

3. **网络 / 非本地路径（KIO）**  
   KDE 侧栏可进 `smb://`、`sftp://`、trash、远程书签。Omarchy 只走本地 `file://`。**刻意不做整棵 KIO 栈**，若做也只考虑有明确用户需求的子集。

4. **列表语义更接近 KFileWidget**（可选）  
   目录优先排序、真·列视图细节、KDE Places（含远程/设备）。

5. **Recent 数据源差异**（可选）  
   Omarchy：`recently-used.xbel`；KDE：recent-docs / KFilePlaces。不必强行统一。

6. **choices 控件形态**（可选）  
   协议已齐；视觉是自绘一行，不是嵌在 KFileWidget 底部的原生控件。

## 刻意不必对齐

- **SaveFiles**：规范有，KDE 桌面端基本不实现；Omarchy 已有，算超集。

## 自测

```bash
python3 scripts/portal-call.py open
python3 scripts/portal-call.py save
python3 scripts/portal-call.py open-dir
cargo run -- --demo file-chooser   # 无 D-Bus egui 后备
```
