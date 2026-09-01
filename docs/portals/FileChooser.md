# FileChooser

状态：**已实现**（日常本机 Open/Save 可用）  
源码：`src/portals/file_chooser.rs`、`src/documents.rs`、`shell/omarchy.portal/FileChooserDialog.qml`、`src/ui/file_chooser.rs`（egui 后备）

## 已完成

- Open / Save / SaveFiles
- Places：Recent（`recently-used.xbel`）、Home、XDG 用户目录、Computer=`/`、GTK bookmarks；Home 别名去重
- 过滤器：portal glob + MIME 展开；结果带 `current_filter`
- 新文件夹、隐藏文件、面包屑（末段可编辑，Ctrl+L 编完整路径）、搜索、覆写确认
- 列表按 Name / Size / Modified 排序；**目录优先**（QML `showDirsFirst` + egui）
- 图片行缩略图 / 其它 MIME 字形
- Save：按当前过滤补扩展名；`current_file` 只给 basename 时预填；列表预选已存在的 `current_file`；SaveFiles 生成不冲突文件名
- `choices`：开关或下拉；Open 结果带 `writable=true`
- 预览：选中已存在的图片或文本时显示侧栏
- UI：`Color.popups` / `Color.menu` 选中态，与 launcher 一致
- **沙箱路径还原（Documents）**：`/run/user/…/doc/…` → `GetMountPoint` + `Info` 宿主路径；相对 `current_folder` 忽略；不存在的绝对路径上溯父目录

## 延后

1. **`parent_window` + `modal`**（跨 portal）  
   当前独立 `FloatingWindow`，未附着调用方窗口。

2. **网络 / 非本地路径**  
   smb/sftp/trash、远程书签。不做远程 VFS。

3. **出站沙箱路径重挂**  
   只做 Documents 入向还原，不把返回 URI 再挂回沙箱 fuse。

4. **Recent 数据源 / choices 视觉**（可选）  
   目前用 `recently-used.xbel`；choices 自绘一行即可。

## 超集

- **SaveFiles**：规范有；本后端已实现。

## 自测

```bash
cargo test --lib documents
python3 scripts/portal-call.py open
python3 scripts/portal-call.py save
python3 scripts/portal-call.py open-dir
# 沙箱路径（本机已有 document id 时）：
python3 scripts/portal-call.py open --folder /run/user/$UID/doc/<doc_id>
python3 scripts/portal-call.py save --folder ~/Downloads --file ~/Downloads/WeCom_5.0.6.6028.exe
cargo run -- --demo file-chooser   # 无 D-Bus egui 后备
```
