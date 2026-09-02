import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import Qt.labs.folderlistmodel
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

PortalDialog {
  id: root

  property var request: ({})
  property var extra: ({})
  property url folder: folderUrl(request.current_folder || Quickshell.env("HOME") || "/")
  property string filename: String(request.current_name || "")
  property bool saveMode: String(request.mode || "Open") === "Save"
  property bool saveFilesMode: String(request.mode || "Open") === "SaveFiles"
  property bool dirMode: request.directory === true || saveFilesMode
  property bool multiple: request.multiple === true
  property var selectedPaths: []
  property int filterIndex: Number(extra.filterIndex || request.current_filter || 0)
  property string query: ""
  property bool showHidden: false
  property string overwritePath: ""
  property bool newFolderOpen: false
  property string newFolderName: ""
  property string newFolderError: ""
  property string previewText: ""
  property var choiceValues: ({})
  property var folderNames: []
  property bool selectedIsDir: false
  property bool recentMode: false
  property bool pathEditing: false
  property string pathEdit: ""
  property string lastSegmentEdit: ""
  property string sortKey: "name"
  property bool sortReversed: false
  property bool didPreselectCurrentFile: false
  property real crumbBarWidth: 0
  property real crumbWidthBudget: 0
  property var visibleCrumbItems: []
  property var ellipsisMenuItems: []
  readonly property int crumbMaxLabelWidth: Style.space(140)

  title: String(request.title || (saveMode ? "Save" : (dirMode ? "Select folder" : "Open")))
  acceptText: String(request.accept_label || (saveMode || saveFilesMode ? "Save" : (dirMode ? "Select" : "Open")))
  acceptable: canAccept()
  showButtons: false
  cardWidth: Style.space(960)
  cardHeight: Style.space(640)
  focus: true
  readonly property int footerBarHeight: Style.spacing.controlHeight
  readonly property bool hasFooterExtras: root.choices.length > 0 || root.filters.length > 0
  readonly property int rowHeight: Style.spacing.controlHeight
  readonly property int iconColWidth: Style.space(22)
  readonly property int dateColWidth: Style.space(140)
  readonly property int sizeColWidth: Style.space(80)
  readonly property int sidebarWidth: Style.space(200)
  readonly property string recentPlace: "recent:"
  readonly property color contentText: Color.popups.text
  readonly property color dimText: Util.alpha(Color.popups.text, 0.72)
  readonly property color selectedBackground: Color.menu.selectedBackground
  readonly property color selectedText: Color.menu.selectedText
  readonly property var selectedBorderSpec: Border.surfaceSpec("menu", "selected-border", Color.menu.selectedBorder, 0)

  signal picked(var paths, var choices, var currentFilter)

  readonly property var places: extra.places || [
    { label: "Home", path: Quickshell.env("HOME") || "/" }
  ]
  readonly property var recentItems: extra.recent || []
  readonly property var filters: (extra.filters && extra.filters.length) ? extra.filters : requestFilters()
  readonly property var choices: request.choices || []
  readonly property string currentPath: String(folder).replace(/^file:\/\//, "")
  readonly property string previewPath: {
    if (selectedPaths.length && !selectedIsDir)
      return selectedPaths[0]
    return ""
  }
  readonly property bool previewIsDir: selectedIsDir
  readonly property bool previewIsImage: previewPath.length > 0 && isImagePath(previewPath)
  readonly property bool previewIsText: previewPath.length > 0 && isTextPath(previewPath) && !previewIsImage
  readonly property bool previewVisible: previewIsImage || (previewIsText && previewText.length > 0)
  readonly property var crumbs: breadcrumbCrumbs()
  readonly property var recentRows: buildRecentRows()

  function requestFilters() {
    var raw = request.filters || []
    var out = []
    for (var i = 0; i < raw.length; i++) {
      var globs = []
      var portalPats = []
      var pats = raw[i].patterns || []
      for (var j = 0; j < pats.length; j++) {
        if (pats[j].Glob !== undefined) {
          globs.push(pats[j].Glob)
          portalPats.push([0, pats[j].Glob])
        } else if (pats[j].Mime !== undefined) {
          portalPats.push([1, pats[j].Mime])
        }
      }
      out.push({
        label: raw[i].label,
        globs: globs.length ? globs : ["*"],
        portal: [raw[i].label, portalPats]
      })
    }
    return out
  }

  function folderUrl(p) {
    p = String(p || "/")
    if (p.indexOf("file:") === 0) return p
    return "file://" + p
  }

  function canAccept() {
    if (overwritePath.length || newFolderOpen) return false
    if (saveMode) return filename.trim().length > 0
    if (dirMode) return true
    return selectedPaths.length > 0 && !selectedIsDir
  }

  function breadcrumbCrumbs() {
    if (recentMode)
      return [{ label: "Recent", path: recentPlace }]
    var path = currentPath.replace(/\/$/, "") || "/"
    var out = [{ label: "/", path: "/" }]
    if (path === "/")
      return out
    var parts = path.split("/")
    var acc = ""
    for (var i = 1; i < parts.length; i++) {
      if (!parts[i].length) continue
      acc += "/" + parts[i]
      out.push({ label: parts[i], path: acc })
    }
    return out
  }

  function crumbChromeWidth() {
    // Button reserves hover/focus borders + horizontal padding; stay conservative
    // so the fitted model does not overflow the real Row layout.
    return Style.spacing.controlPaddingX * 2 + Style.space(12)
  }

  function measureCrumbLabel(label, maxLabelW, bold) {
    crumbMetrics.font.bold = !!bold
    crumbMetrics.elide = Text.ElideNone
    crumbMetrics.text = String(label)
    var textW = crumbMetrics.width
    if (maxLabelW > 0 && textW > maxLabelW) {
      crumbMetrics.elide = Text.ElideMiddle
      crumbMetrics.elideWidth = maxLabelW
      return {
        text: crumbMetrics.elidedText,
        width: maxLabelW + crumbChromeWidth(),
        truncated: true,
        full: String(label)
      }
    }
    return {
      text: String(label),
      width: textW + crumbChromeWidth(),
      truncated: false,
      full: String(label)
    }
  }

  function rebuildVisibleCrumbs() {
    var all = breadcrumbCrumbs()
    var gap = Style.spacing.xs
    var avail = crumbWidthBudget > 0 ? crumbWidthBudget : crumbBarWidth
    var maxLabelW = crumbMaxLabelWidth
    var measured = []
    for (var i = 0; i < all.length; i++) {
      var selected = i === all.length - 1
      var m = measureCrumbLabel(all[i].label, maxLabelW, selected)
      measured.push({
        kind: "crumb",
        label: m.text,
        fullLabel: m.full,
        path: all[i].path,
        selected: selected,
        width: m.width,
        truncated: m.truncated,
        sourceIndex: i
      })
    }
    if (measured.length <= 2 || avail <= 0) {
      visibleCrumbItems = measured
      return
    }
    var total = 0
    for (var j = 0; j < measured.length; j++)
      total += measured[j].width + (j > 0 ? gap : 0)
    if (total <= avail) {
      visibleCrumbItems = measured
      return
    }

    var ellipsisM = measureCrumbLabel("…", 0, false)
    var ellipsisW = ellipsisM.width
    var first = measured[0]
    var last = measured[measured.length - 1]
    var keepTail = [last]
    var used = first.width + gap + ellipsisW + gap + last.width

    // If even root + … + current overflows, shrink the current label.
    if (used > avail) {
      var over = used - avail
      var shrinkTo = Math.max(Style.space(48), maxLabelW - over)
      var lastRaw = all[all.length - 1]
      var sm = measureCrumbLabel(lastRaw.label, shrinkTo, true)
      last = {
        kind: "crumb",
        label: sm.text,
        fullLabel: sm.full,
        path: lastRaw.path,
        selected: true,
        width: sm.width,
        truncated: sm.truncated,
        sourceIndex: all.length - 1
      }
      keepTail = [last]
      used = first.width + gap + ellipsisW + gap + last.width
    }

    for (var k = measured.length - 2; k >= 1; k--) {
      var nextUsed = used + gap + measured[k].width
      if (nextUsed <= avail) {
        keepTail.unshift(measured[k])
        used = nextUsed
      } else {
        break
      }
    }

    var firstTailIdx = keepTail[0].sourceIndex
    var hidden = []
    for (var h = 1; h < firstTailIdx; h++)
      hidden.push({ label: all[h].label, path: all[h].path })

    if (hidden.length === 0) {
      visibleCrumbItems = [first].concat(keepTail)
      return
    }

    var items = [
      first,
      { kind: "ellipsis", label: "…", hidden: hidden, width: ellipsisW }
    ]
    for (var t = 0; t < keepTail.length; t++)
      items.push(keepTail[t])
    visibleCrumbItems = items
  }

  function ensureCrumbsFit() {
    if (!crumbBarWidth || crumbBarWidth <= 0)
      return
    var rowW = crumbRow ? crumbRow.implicitWidth : 0
    if (rowW <= crumbBarWidth + 1)
      return
    // Actual buttons were wider than the estimate — tighten budget and refit.
    var tighter = Math.max(Style.space(120), 2 * crumbBarWidth - rowW - Style.space(8))
    if (tighter >= crumbWidthBudget && crumbWidthBudget > 0)
      tighter = Math.max(Style.space(120), crumbWidthBudget - Style.space(24))
    if (Math.abs(tighter - crumbWidthBudget) < 1)
      return
    crumbWidthBudget = tighter
    rebuildVisibleCrumbs()
  }

  function openEllipsisMenu(hidden, anchorItem) {
    ellipsisMenuItems = hidden || []
    if (!ellipsisMenuItems.length)
      return
    ellipsisPopup.anchorItem = anchorItem
    ellipsisPopup.open()
  }

  onCrumbsChanged: rebuildVisibleCrumbs()

  function syncPathEdits() {
    pathEdit = recentMode ? recentPlace : currentPath
    var c = breadcrumbCrumbs()
    lastSegmentEdit = c.length ? String(c[c.length - 1].label) : ""
  }

  function goTo(path) {
    var p = String(path || "")
    if (!p) return
    if (p === recentPlace) {
      recentMode = true
      pathEditing = false
      selectedPaths = []
      selectedIsDir = false
      previewText = ""
      syncPathEdits()
      return
    }
    recentMode = false
    pathEditing = false
    root.folder = folderUrl(p)
    root.selectedPaths = []
    root.selectedIsDir = false
    root.previewText = ""
    syncPathEdits()
  }

  function goParent() {
    if (recentMode) {
      goTo(Quickshell.env("HOME") || "/")
      return
    }
    var path = currentPath.replace(/\/$/, "")
    if (path === "" || path === "/") return
    var parent = path.replace(/\/[^\/]+$/, "")
    goTo(parent.length ? parent : "/")
  }

  function commitPathEdit(text) {
    var p = String(text || "").trim()
    if (!p) return
    if (p === recentPlace) {
      goTo(recentPlace)
      return
    }
    if (p.charAt(0) !== "/") p = "/" + p
    goTo(p)
  }

  function commitLastSegment(text) {
    var name = String(text || "").trim()
    if (!name.length) return
    if (recentMode || name.charAt(0) === "/" || name === recentPlace) {
      commitPathEdit(name)
      return
    }
    if (name === "..") {
      goParent()
      return
    }
    var c = crumbs
    if (c.length <= 1) {
      goTo("/" + name)
      return
    }
    var parent = String(c[c.length - 2].path)
    if (parent === "/")
      goTo("/" + name)
    else
      goTo(parent + "/" + name)
  }

  function currentGlobs() {
    if (!filters.length) return []
    var i = Math.max(0, Math.min(filterIndex, filters.length - 1))
    return filters[i].globs || []
  }

  function globMatch(glob, name) {
    if (glob === "*" || glob === "*.*") return true
    var re = String(glob).replace(/[.+^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*").replace(/\?/g, ".")
    return new RegExp("^" + re + "$", "i").test(name)
  }

  function matchesFilter(name, isDir) {
    if (isDir) return true
    var g = currentGlobs()
    if (!g.length || (g.length === 1 && g[0] === "*")) return true
    for (var i = 0; i < g.length; i++) {
      if (globMatch(g[i], name)) return true
    }
    return false
  }

  function currentFilterPortal() {
    if (!filters.length) return null
    var i = Math.max(0, Math.min(filterIndex, filters.length - 1))
    return filters[i].portal || null
  }

  function selectedChoices() {
    var out = []
    var list = choices
    for (var i = 0; i < list.length; i++) {
      var id = String(list[i].id || "")
      var val = choiceValues[id]
      if (val === undefined) val = list[i].selected
      out.push([id, String(val || "")])
    }
    return out
  }

  function applyDefaultExtension(name) {
    var g = currentGlobs()
    if (g.length !== 1) return name
    var glob = String(g[0] || "")
    if (glob.indexOf("*.") !== 0 || glob.indexOf("*", 1) !== -1) return name
    var ext = glob.slice(1)
    if (!ext || ext === ".*") return name
    if (name.toLowerCase().endsWith(ext.toLowerCase())) return name
    if (name.indexOf(".") !== -1) return name
    return name + ext
  }

  function savePath() {
    var dir = currentPath.replace(/\/$/, "")
    return dir + "/" + applyDefaultExtension(filename.trim())
  }

  function nameInFolder(name) {
    return folderNames.indexOf(name) !== -1
  }

  function uniqueInFolder(dir, name) {
    if (!nameInFolder(name)) return dir + "/" + name
    var dot = name.lastIndexOf(".")
    var stem = dot > 0 ? name.slice(0, dot) : name
    var ext = dot > 0 ? name.slice(dot) : ""
    for (var n = 1; n < 10000; n++) {
      var candidate = stem + " (" + n + ")" + ext
      if (!nameInFolder(candidate)) return dir + "/" + candidate
    }
    return dir + "/" + name
  }

  function emitPicked(paths) {
    root.picked(paths, selectedChoices(), currentFilterPortal())
  }

  function tryAccept() {
    if (overwritePath.length || newFolderOpen) return
    if (saveMode) {
      var name = applyDefaultExtension(filename.trim())
      if (!name.length) return
      root.filename = name
      var path = savePath()
      if (nameInFolder(name) || selectedPaths.indexOf(path) !== -1) {
        overwritePath = path
        return
      }
      existsProc.checkPath = path
      existsProc.running = false
      existsProc.running = true
      return
    }
    if (saveFilesMode) {
      var dir = currentPath.replace(/\/$/, "")
      var names = request.save_names || []
      var paths = []
      if (!names.length && filename.trim().length)
        names = [filename.trim()]
      if (!names.length) {
        emitPicked([dir])
        return
      }
      for (var i = 0; i < names.length; i++)
        paths.push(uniqueInFolder(dir, String(names[i])))
      emitPicked(paths)
      return
    }
    if (dirMode) {
      if (selectedPaths.length && selectedIsDir)
        emitPicked(selectedPaths.slice())
      else if (!recentMode)
        emitPicked([currentPath])
      return
    }
    if (selectedPaths.length === 1 && selectedIsDir) {
      goTo(selectedPaths[0])
      return
    }
    if (selectedPaths.length)
      emitPicked(selectedPaths.slice())
  }

  function toggleSelect(path, isDir, name) {
    root.selectedIsDir = isDir === true
    if (isDir && !dirMode) {
      root.selectedPaths = [path]
      return
    }
    if (root.multiple && !saveMode) {
      var next = root.selectedPaths.slice()
      var i = next.indexOf(path)
      if (i >= 0) next.splice(i, 1)
      else next.push(path)
      root.selectedPaths = next
    } else {
      root.selectedPaths = [path]
    }
    if ((saveMode || saveFilesMode) && name && !isDir)
      root.filename = name
  }

  function matchesQuery(name) {
    if (!query.length) return true
    return String(name).toLowerCase().indexOf(query.toLowerCase()) !== -1
  }

  function rowVisible(name, isDir) {
    return matchesQuery(name) && matchesFilter(name, isDir)
  }

  function isImagePath(path) {
    return /\.(png|jpe?g|gif|webp|bmp|svg|tif|tiff|avif|jxl|heic|ico)$/i.test(path)
  }

  function isTextPath(path) {
    return /\.(txt|md|csv|json|xml|html?|css|js|ts|rs|py|toml|ini|log|conf|yml|yaml)$/i.test(path)
  }

  function formatSize(bytes) {
    bytes = Number(bytes || 0)
    if (bytes < 1024) return bytes + " B"
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(0) + " KB"
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MB"
    return (bytes / (1024 * 1024 * 1024)).toFixed(1) + " GB"
  }

  function formatDate(value) {
    if (value === undefined || value === null || value === "") return ""
    var date
    if (value instanceof Date)
      date = value
    else if (typeof value === "number")
      date = value > 1e12 ? new Date(value) : new Date(value * 1000)
    else
      date = new Date(value)
    if (isNaN(date.getTime())) return ""
    function pad(n) { return n < 10 ? "0" + n : "" + n }
    return date.getFullYear() + "-" + pad(date.getMonth() + 1) + "-" + pad(date.getDate())
      + " " + pad(date.getHours()) + ":" + pad(date.getMinutes())
  }

  function placeGlyph(label, path) {
    if (String(path) === recentPlace) return "󰋚"
    switch (String(label)) {
    case "Home": return "󰋜"
    case "Downloads": return "󰇚"
    case "Documents": return "󰈙"
    case "Pictures": return "󰉏"
    case "Videos": return "󰎁"
    case "Music": return "󰎆"
    case "Projects": return "󰲂"
    case "Computer": return "󰌢"
    default: return "󰉋"
    }
  }

  function fileGlyph(isDir, name) {
    if (isDir) return "󰉋"
    if (isImagePath(name)) return "󰋩"
    if (isTextPath(name)) return "󰈙"
    return "󰈔"
  }

  function placeSelected(path) {
    if (String(path) === recentPlace) return recentMode
    if (recentMode) return false
    var a = String(path).replace(/\/$/, "") || "/"
    var b = currentPath.replace(/\/$/, "") || "/"
    return a === b
  }

  function cycleSort(key) {
    if (sortKey === key)
      sortReversed = !sortReversed
    else {
      sortKey = key
      sortReversed = false
    }
    applyFolderSort()
  }

  function applyFolderSort() {
    if (sortKey === "size")
      files.sortField = FolderListModel.Size
    else if (sortKey === "time")
      files.sortField = FolderListModel.Time
    else
      files.sortField = FolderListModel.Name
    files.sortReversed = sortReversed
    // Keep directories pinned; Qt resets this when sortField changes on some versions.
    files.showDirsFirst = true
  }

  function tryPreselectCurrentFile() {
    if (root.didPreselectCurrentFile || root.recentMode)
      return
    var want = String(request.current_file || "")
    if (!want.length)
      return
    var folderPath = root.currentPath
    var parent = want.replace(/\/[^\/]+$/, "")
    if (parent !== folderPath)
      return
    var name = want.split("/").pop()
    root.selectedPaths = [want]
    root.selectedIsDir = false
    if ((root.saveMode || root.saveFilesMode) && name)
      root.filename = name
    root.didPreselectCurrentFile = true
  }

  function sortHeading(label, key) {
    if (sortKey !== key) return label
    return label + (sortReversed ? " ↓" : " ↑")
  }

  function buildRecentRows() {
    var rows = (recentItems || []).slice()
    var q = query
    var hidden = showHidden
    var key = sortKey
    var rev = sortReversed
    rows = rows.filter(function(row) {
      var name = String(row.label || "")
      var isDir = row.isDir === true
      if (!hidden && name.charAt(0) === ".") return false
      return rowVisible(name, isDir)
    })
    rows.sort(function(a, b) {
      var aDir = a.isDir === true
      var bDir = b.isDir === true
      if (aDir !== bDir) return aDir ? -1 : 1
      var cmp = 0
      if (key === "size")
        cmp = Number(a.size || 0) - Number(b.size || 0)
      else if (key === "time")
        cmp = Number(a.modified || 0) - Number(b.modified || 0)
      else
        cmp = String(a.label || "").toLowerCase().localeCompare(String(b.label || "").toLowerCase())
      return rev ? -cmp : cmp
    })
    return rows
  }

  function refreshList() {
    var f = files.folder
    files.folder = "file:///proc/self"
    files.folder = f
    refreshNames()
  }

  function refreshNames() {
    if (recentMode) return
    lsProc.running = false
    lsProc.running = true
  }

  function createFolder() {
    var name = newFolderName.trim()
    if (!name.length || name.indexOf("/") !== -1) {
      newFolderError = "Enter a folder name"
      return
    }
    if (nameInFolder(name)) {
      newFolderError = "A folder with that name already exists"
      return
    }
    mkdirProc.dirPath = currentPath.replace(/\/$/, "") + "/" + name
    mkdirProc.running = false
    mkdirProc.running = true
  }

  function choiceHasOptions(choice) {
    return choice && choice.options && choice.options.length
  }

  // Same as AccessDialog: QVariant pairs may be objects with "0"/"1" keys.
  function optionPair(o, index) {
    if (o === undefined || o === null)
      return { value: String(index), label: String(index) }
    if (Array.isArray(o))
      return { value: String(o[0]), label: String(o[1] || o[0]) }
    if (typeof o === "object") {
      if (o.value !== undefined || o.label !== undefined || o.id !== undefined) {
        var v = o.value !== undefined ? o.value : (o.id !== undefined ? o.id : index)
        var l = o.label !== undefined ? o.label : v
        return { value: String(v), label: String(l) }
      }
      if (o[0] !== undefined || o["0"] !== undefined) {
        var pv = o[0] !== undefined ? o[0] : o["0"]
        var pl = o[1] !== undefined ? o[1] : o["1"]
        return { value: String(pv), label: String(pl !== undefined ? pl : pv) }
      }
    }
    return { value: String(o), label: String(o) }
  }

  function choiceOptions(choice) {
    var out = []
    var opts = (choice && choice.options) || []
    for (var i = 0; i < opts.length; i++)
      out.push(optionPair(opts[i], i))
    return out
  }

  onAccepted: tryAccept()
  onFolderChanged: {
    if (!recentMode)
      refreshNames()
    if (!pathEditing)
      syncPathEdits()
  }
  Component.onCompleted: {
    refreshNames()
    syncPathEdits()
    applyFolderSort()
    var initial = {}
    for (var i = 0; i < choices.length; i++) {
      var c = choices[i]
      var fallback = "false"
      if (c.options && c.options.length)
        fallback = optionPair(c.options[0], 0).value
      initial[String(c.id || i)] = String(c.selected || fallback)
    }
    choiceValues = initial
    tryPreselectCurrentFile()
    rebuildVisibleCrumbs()
  }

  TextMetrics {
    id: crumbMetrics
    font.family: Style.font.family
    font.pixelSize: Style.font.body
  }

  onPreviewPathChanged: {
    previewText = ""
    if (previewIsText && previewPath.length) {
      headProc.filePath = previewPath
      headProc.running = false
      headProc.running = true
    }
  }

  FolderListModel {
    id: files
    folder: root.folder
    showDirs: true
    showDirsFirst: true
    showFiles: !root.dirMode || root.saveMode
    showDotAndDotDot: false
    showHidden: root.showHidden
    sortField: FolderListModel.Name
    sortReversed: false
  }

  Process {
    id: lsProc
    command: ["ls", "-1A", "--", root.currentPath]
    stdout: StdioCollector {
      onStreamFinished: {
        root.folderNames = String(text || "").split("\n").filter(function(s) { return s.length > 0 })
      }
    }
  }

  Process {
    id: existsProc
    property string checkPath: ""
    command: ["test", "-e", checkPath]
    onExited: function(code) {
      if (code === 0) root.overwritePath = checkPath
      else root.emitPicked([checkPath])
    }
  }

  Process {
    id: mkdirProc
    property string dirPath: ""
    command: ["mkdir", "--", dirPath]
    onExited: function(code) {
      if (code === 0) {
        var created = dirPath
        root.newFolderOpen = false
        root.newFolderName = ""
        root.newFolderError = ""
        root.refreshList()
        root.goTo(created)
      } else {
        root.newFolderError = "Could not create folder"
      }
    }
  }

  Process {
    id: headProc
    property string filePath: ""
    command: ["head", "-c", "1200", filePath]
    stdout: StdioCollector {
      onStreamFinished: root.previewText = text
    }
  }

  function handleKey(event) {
    if (overwriteDlg.opened)
      return overwriteDlg.handleKey(event)
    if (newFolderOpen) {
      if (event.key === Qt.Key_Escape) {
        newFolderOpen = false
        return true
      }
      if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
        createFolder()
        return true
      }
      return false
    }
    if (pathEditing && event.key === Qt.Key_Escape) {
      pathEditing = false
      syncPathEdits()
      return true
    }
    if (event.key === Qt.Key_L && (event.modifiers & Qt.ControlModifier)) {
      pathEditing = true
      pathEdit = recentMode ? recentPlace : currentPath
      return true
    }
    if (event.key === Qt.Key_Backspace && !(event.modifiers & Qt.ControlModifier)) {
      goParent()
      return true
    }
    if (event.key === Qt.Key_Escape) {
      root.rejected()
      return true
    }
    if (event.key === Qt.Key_Left || event.key === Qt.Key_Right || event.key === Qt.Key_Tab || event.key === Qt.Key_Backtab) {
      root.selectedIndex = root.selectedIndex === 0 ? 1 : 0
      return true
    }
    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
      if (root.selectedIndex === 0) root.rejected()
      else if (root.acceptable) root.accepted()
      return true
    }
    return false
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.spacing.lg

    RowLayout {
      Layout.fillWidth: true
      Layout.fillHeight: true
      spacing: Style.spacing.lg

      ListView {
        id: placeList
        Layout.preferredWidth: root.sidebarWidth
        Layout.fillHeight: true
        clip: true
        spacing: Style.spacing.xs
        model: root.places
        delegate: BorderSurface {
          id: placeRow
          required property var modelData
          width: placeList.width
          height: root.rowHeight
          radius: Style.cornerRadius
          readonly property bool sel: root.placeSelected(modelData.path)
          readonly property bool hot: sel || placeHover.containsMouse
          color: hot ? root.selectedBackground : "transparent"
          borderSpec: hot ? root.selectedBorderSpec : Border.none()

          Row {
            anchors.fill: parent
            anchors.leftMargin: Style.spacing.rowPaddingX
            anchors.rightMargin: Style.spacing.rowPaddingX
            spacing: Style.spacing.sm

            Text {
              anchors.verticalCenter: parent.verticalCenter
              text: root.placeGlyph(modelData.label, modelData.path)
              color: placeRow.hot ? root.selectedText : root.contentText
              font.family: Style.font.family
              font.pixelSize: Style.font.icon
              width: root.iconColWidth
              horizontalAlignment: Text.AlignHCenter
            }
            Text {
              anchors.verticalCenter: parent.verticalCenter
              width: parent.width - root.iconColWidth - parent.spacing
              text: String(modelData.label || modelData.path)
              color: placeRow.hot ? root.selectedText : root.contentText
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              elide: Text.ElideRight
            }
          }

          MouseArea {
            id: placeHover
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.goTo(modelData.path)
          }
        }
      }

      ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: Style.spacing.sm

        RowLayout {
          Layout.fillWidth: true
          Layout.preferredHeight: root.rowHeight
          spacing: Style.spacing.sm

          Button {
            iconText: "󰁍"
            bordered: true
            tooltipText: "Parent folder"
            Layout.preferredHeight: root.rowHeight
            onClicked: root.goParent()
          }
          TextField {
            visible: root.pathEditing
            Layout.fillWidth: true
            Layout.preferredHeight: root.rowHeight
            text: root.pathEdit
            onTextChanged: root.pathEdit = text
            onAccepted: root.commitPathEdit(text)
            Keys.onPressed: function(event) {
              if (event.key === Qt.Key_Escape) {
                root.pathEditing = false
                root.syncPathEdits()
                event.accepted = true
              }
            }
          }
          Item {
            id: crumbBar
            visible: !root.pathEditing
            Layout.fillWidth: true
            Layout.preferredHeight: root.rowHeight
            clip: true
            function syncWidth() {
              root.crumbBarWidth = width
              root.crumbWidthBudget = width
              root.rebuildVisibleCrumbs()
              Qt.callLater(root.ensureCrumbsFit)
            }
            onWidthChanged: syncWidth()
            Component.onCompleted: syncWidth()

            Row {
              id: crumbRow
              anchors.left: parent.left
              anchors.verticalCenter: parent.verticalCenter
              height: parent.height
              spacing: Style.spacing.xs
              onImplicitWidthChanged: Qt.callLater(root.ensureCrumbsFit)

              Repeater {
                model: root.visibleCrumbItems
                delegate: Item {
                  id: crumbDelegate
                  required property int index
                  required property var modelData
                  height: crumbRow.height
                  width: crumbButton.width

                  Button {
                    id: crumbButton
                    anchors.verticalCenter: parent.verticalCenter
                    height: crumbDelegate.height
                    text: String(modelData.label)
                    bordered: true
                    selected: modelData.kind === "crumb" && modelData.selected === true
                    tooltipText: {
                      if (modelData.kind === "ellipsis")
                        return "Ancestor folders"
                      if (modelData.truncated)
                        return String(modelData.fullLabel || modelData.label)
                      return ""
                    }
                    onClicked: {
                      if (modelData.kind === "ellipsis") {
                        root.openEllipsisMenu(modelData.hidden, crumbButton)
                        return
                      }
                      if (modelData.selected) {
                        root.pathEditing = true
                        root.pathEdit = root.recentMode ? root.recentPlace : root.currentPath
                      } else {
                        root.goTo(modelData.path)
                      }
                    }
                  }
                }
              }
            }
          }
          TextField {
            Layout.preferredWidth: Style.space(200)
            Layout.preferredHeight: root.rowHeight
            placeholderText: "Search"
            onTextChanged: root.query = text
          }
          Button {
            text: "New folder"
            bordered: true
            enabled: !root.recentMode
            Layout.preferredHeight: root.rowHeight
            onClicked: {
              root.newFolderName = ""
              root.newFolderError = ""
              root.newFolderOpen = true
            }
          }
          Button {
            text: "Hidden"
            bordered: true
            selected: root.showHidden
            Layout.preferredHeight: root.rowHeight
            onClicked: root.showHidden = !root.showHidden
          }
        }

        RowLayout {
          Layout.fillWidth: true
          Layout.fillHeight: true
          spacing: Style.spacing.lg

          ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            RowLayout {
              Layout.fillWidth: true
              Layout.preferredHeight: root.rowHeight
              spacing: Style.spacing.sm

              Item { Layout.preferredWidth: root.iconColWidth }
              Text {
                Layout.fillWidth: true
                text: root.sortHeading("Name", "name")
                color: root.dimText
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                font.bold: true
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.cycleSort("name") }
              }
              Text {
                Layout.preferredWidth: root.sizeColWidth
                horizontalAlignment: Text.AlignRight
                text: root.sortHeading("Size", "size")
                color: root.dimText
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                font.bold: true
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.cycleSort("size") }
              }
              Text {
                Layout.preferredWidth: root.dateColWidth
                horizontalAlignment: Text.AlignRight
                text: root.sortHeading("Modified", "time")
                color: root.dimText
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                font.bold: true
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.cycleSort("time") }
              }
            }
            PanelSeparator { Layout.fillWidth: true }

            ListView {
              id: list
              visible: !root.recentMode
              Layout.fillWidth: true
              Layout.fillHeight: true
              clip: true
              model: files
              delegate: fileRow
            }
            ListView {
              id: recentList
              visible: root.recentMode
              Layout.fillWidth: true
              Layout.fillHeight: true
              clip: true
              model: root.recentRows
              delegate: Item {
                width: recentList.width
                required property var modelData
                property string fileName: String(modelData.label || "")
                property bool fileIsDir: modelData.isDir === true
                property var fileSize: modelData.size || 0
                property var fileModified: modelData.modified || 0
                readonly property string path: String(modelData.path || "")
                readonly property bool sel: root.selectedPaths.indexOf(path) !== -1
                height: root.rowVisible(fileName, fileIsDir) ? root.rowHeight : 0
                visible: height > 0
                FileRowInner {
                  anchors.fill: parent
                  fileName: parent.fileName
                  fileIsDir: parent.fileIsDir
                  fileSize: parent.fileSize
                  fileModified: parent.fileModified
                  path: parent.path
                  sel: parent.sel
                }
              }
            }
          }

          BorderSurface {
            visible: root.previewVisible
            Layout.preferredWidth: Style.space(220)
            Layout.fillHeight: true
            color: "transparent"
            borderSpec: Border.flat(Util.alpha(root.contentText, 0.28), Style.normalBorderWidth)
            radius: Style.cornerRadius
            padding: Style.space(10)

            Column {
              anchors.fill: parent
              anchors.topMargin: parent.contentTopInset
              anchors.rightMargin: parent.contentRightInset
              anchors.bottomMargin: parent.contentBottomInset
              anchors.leftMargin: parent.contentLeftInset
              spacing: Style.spacing.sm

              Text {
                width: parent.width
                text: root.previewPath.length ? root.previewPath.split("/").pop() : "Preview"
                color: root.contentText
                font.family: Style.font.family
                font.pixelSize: Style.font.body
                elide: Text.ElideMiddle
              }
              Image {
                width: parent.width
                height: Math.min(Style.space(160), parent.height - Style.space(72))
                visible: root.previewIsImage
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                source: root.previewIsImage ? "file://" + root.previewPath : ""
                sourceSize.width: 400
                sourceSize.height: 400
              }
              Text {
                width: parent.width
                visible: root.previewIsText
                text: root.previewText.length ? root.previewText : "…"
                color: root.dimText
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                wrapMode: Text.WrapAnywhere
                maximumLineCount: 14
                elide: Text.ElideRight
              }
            }
          }
        }
      }
    }

    PanelSeparator { Layout.fillWidth: true }

    // Open: extras + buttons on a single row.
    RowLayout {
      visible: !root.saveMode
      Layout.fillWidth: true
      Layout.preferredHeight: root.footerBarHeight
      spacing: Style.spacing.sm

      Repeater {
        model: root.choices
        delegate: Row {
          required property var modelData
          required property int index
          spacing: Style.spacing.sm
          Layout.alignment: Qt.AlignVCenter
          Layout.preferredHeight: root.footerBarHeight

          Text {
            visible: root.choiceHasOptions(modelData)
            anchors.verticalCenter: parent.verticalCenter
            text: String(modelData.label || modelData.id)
            color: root.contentText
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          UpDropdown {
            visible: root.choiceHasOptions(modelData)
            width: Style.space(160)
            height: root.footerBarHeight
            showLabel: false
            rowHeight: root.footerBarHeight
            value: String(root.choiceValues[String(modelData.id || index)] || modelData.selected || "")
            options: root.choiceOptions(modelData)
            onChanged: function(v) {
              var next = Object.assign({}, root.choiceValues)
              next[String(modelData.id || index)] = v
              root.choiceValues = next
            }
          }
          Text {
            visible: !root.choiceHasOptions(modelData)
            anchors.verticalCenter: parent.verticalCenter
            text: String(modelData.label || modelData.id)
            color: root.contentText
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          ToggleSwitch {
            visible: !root.choiceHasOptions(modelData)
            anchors.verticalCenter: parent.verticalCenter
            checked: String(root.choiceValues[String(modelData.id || index)] || modelData.selected) === "true"
            onToggled: {
              var next = Object.assign({}, root.choiceValues)
              next[String(modelData.id || index)] = checked ? "false" : "true"
              root.choiceValues = next
            }
          }
        }
      }
      UpDropdown {
        visible: root.filters.length > 0
        Layout.preferredWidth: Style.space(200)
        Layout.preferredHeight: root.footerBarHeight
        Layout.alignment: Qt.AlignVCenter
        showLabel: false
        rowHeight: root.footerBarHeight
        value: String(root.filterIndex)
        options: {
          var out = []
          for (var i = 0; i < root.filters.length; i++)
            out.push({ value: String(i), label: String(root.filters[i].label || "Filter") })
          return out
        }
        onChanged: function(v) { root.filterIndex = Number(v) }
      }
      Item { Layout.fillWidth: true }
      Button {
        text: root.cancelText
        bordered: true
        selected: root.selectedIndex === 0
        Layout.preferredHeight: root.footerBarHeight
        Layout.alignment: Qt.AlignVCenter
        onClicked: {
          root.selectedIndex = 0
          root.rejected()
        }
      }
      Button {
        text: root.acceptText
        bordered: true
        selected: root.selectedIndex === 1
        enabled: root.acceptable
        Layout.preferredHeight: root.footerBarHeight
        Layout.alignment: Qt.AlignVCenter
        onClicked: {
          root.selectedIndex = 1
          if (root.acceptable) root.accepted()
        }
      }
    }

    // Save: extras row, then filename + Cancel/Save.
    RowLayout {
      visible: root.saveMode && root.hasFooterExtras
      Layout.fillWidth: true
      Layout.preferredHeight: root.footerBarHeight
      spacing: Style.spacing.md

      Repeater {
        model: root.choices
        delegate: Row {
          required property var modelData
          required property int index
          spacing: Style.spacing.sm
          Layout.alignment: Qt.AlignVCenter
          Layout.preferredHeight: root.footerBarHeight

          Text {
            visible: root.choiceHasOptions(modelData)
            anchors.verticalCenter: parent.verticalCenter
            text: String(modelData.label || modelData.id)
            color: root.contentText
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          UpDropdown {
            visible: root.choiceHasOptions(modelData)
            width: Style.space(160)
            height: root.footerBarHeight
            showLabel: false
            rowHeight: root.footerBarHeight
            value: String(root.choiceValues[String(modelData.id || index)] || modelData.selected || "")
            options: root.choiceOptions(modelData)
            onChanged: function(v) {
              var next = Object.assign({}, root.choiceValues)
              next[String(modelData.id || index)] = v
              root.choiceValues = next
            }
          }
          Text {
            visible: !root.choiceHasOptions(modelData)
            anchors.verticalCenter: parent.verticalCenter
            text: String(modelData.label || modelData.id)
            color: root.contentText
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
          ToggleSwitch {
            visible: !root.choiceHasOptions(modelData)
            anchors.verticalCenter: parent.verticalCenter
            checked: String(root.choiceValues[String(modelData.id || index)] || modelData.selected) === "true"
            onToggled: {
              var next = Object.assign({}, root.choiceValues)
              next[String(modelData.id || index)] = checked ? "false" : "true"
              root.choiceValues = next
            }
          }
        }
      }
      UpDropdown {
        visible: root.filters.length > 0
        Layout.preferredWidth: Style.space(200)
        Layout.preferredHeight: root.footerBarHeight
        Layout.alignment: Qt.AlignVCenter
        showLabel: false
        rowHeight: root.footerBarHeight
        value: String(root.filterIndex)
        options: {
          var out = []
          for (var i = 0; i < root.filters.length; i++)
            out.push({ value: String(i), label: String(root.filters[i].label || "Filter") })
          return out
        }
        onChanged: function(v) { root.filterIndex = Number(v) }
      }
      Item { Layout.fillWidth: true }
    }
    RowLayout {
      visible: root.saveMode
      Layout.fillWidth: true
      Layout.preferredHeight: root.footerBarHeight
      spacing: Style.spacing.sm

      TextField {
        Layout.fillWidth: true
        Layout.preferredHeight: root.footerBarHeight
        placeholderText: "File name"
        text: root.filename
        onTextChanged: root.filename = text
        onAccepted: root.tryAccept()
      }
      Button {
        text: root.cancelText
        bordered: true
        selected: root.selectedIndex === 0
        Layout.preferredHeight: root.footerBarHeight
        onClicked: {
          root.selectedIndex = 0
          root.rejected()
        }
      }
      Button {
        text: root.acceptText
        bordered: true
        selected: root.selectedIndex === 1
        enabled: root.acceptable
        Layout.preferredHeight: root.footerBarHeight
        onClicked: {
          root.selectedIndex = 1
          if (root.acceptable) root.accepted()
        }
      }
    }
  }

  Component {
    id: fileRow
    Item {
      width: list.width
      height: root.rowVisible(fileName, fileIsDir) ? root.rowHeight : 0
      visible: height > 0
      property string fileName: model.fileName || ""
      property bool fileIsDir: model.fileIsDir === true
      property var fileSize: model.fileSize || 0
      property var fileModified: model.fileModified
      readonly property string path: String(model.filePath || model.fileURL || "").replace(/^file:\/\//, "")
      readonly property bool sel: root.selectedPaths.indexOf(path) !== -1
      FileRowInner {
        anchors.fill: parent
        fileName: parent.fileName
        fileIsDir: parent.fileIsDir
        fileSize: parent.fileSize
        fileModified: parent.fileModified
        path: parent.path
        sel: parent.sel
      }
    }
  }

  component FileRowInner: Item {
    id: row
    property string fileName: ""
    property bool fileIsDir: false
    property var fileSize: 0
    property var fileModified
    property string path: ""
    property bool sel: false
    readonly property bool isImage: !fileIsDir && root.isImagePath(fileName)
    readonly property bool hot: sel || rowHover.containsMouse
    readonly property color rowText: hot ? root.selectedText : root.contentText
    readonly property color rowDim: hot ? Util.alpha(root.selectedText, 0.7) : root.dimText

    BorderSurface {
      anchors.fill: parent
      radius: Style.cornerRadius
      color: row.hot ? root.selectedBackground : "transparent"
      borderSpec: row.hot ? root.selectedBorderSpec : Border.none()
    }

    RowLayout {
      anchors.fill: parent
      anchors.leftMargin: Style.spacing.sm
      anchors.rightMargin: Style.spacing.sm
      spacing: Style.spacing.sm

      Item {
        Layout.preferredWidth: root.iconColWidth
        Layout.preferredHeight: root.iconColWidth
        Text {
          anchors.centerIn: parent
          visible: !row.isImage || thumb.status !== Image.Ready
          text: root.fileGlyph(row.fileIsDir, row.fileName)
          color: row.rowText
          font.family: Style.font.family
          font.pixelSize: Style.font.icon
        }
        Image {
          id: thumb
          anchors.fill: parent
          visible: row.isImage
          fillMode: Image.PreserveAspectCrop
          asynchronous: true
          source: row.isImage ? "file://" + row.path : ""
          sourceSize.width: 48
          sourceSize.height: 48
        }
      }
      Text {
        Layout.fillWidth: true
        text: row.fileName
        color: row.rowText
        font.family: Style.font.family
        font.pixelSize: Style.font.body
        elide: Text.ElideRight
      }
      Text {
        Layout.preferredWidth: root.sizeColWidth
        horizontalAlignment: Text.AlignRight
        text: row.fileIsDir ? "—" : root.formatSize(row.fileSize)
        color: row.rowDim
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
      Text {
        Layout.preferredWidth: root.dateColWidth
        horizontalAlignment: Text.AlignRight
        text: row.fileIsDir ? "—" : root.formatDate(row.fileModified)
        color: row.rowDim
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
    }

    MouseArea {
      id: rowHover
      anchors.fill: parent
      hoverEnabled: true
      cursorShape: Qt.PointingHandCursor
      onClicked: root.toggleSelect(row.path, row.fileIsDir, row.fileName)
      onDoubleClicked: {
        if (row.fileIsDir) {
          root.goTo(row.path)
        } else if (root.saveMode) {
          root.filename = row.fileName
          if (root.recentMode) {
            var parent = String(row.path).replace(/\/[^\/]+$/, "")
            root.goTo(parent.length ? parent : "/")
            root.filename = row.fileName
          }
          root.tryAccept()
        } else {
          root.selectedPaths = [row.path]
          root.tryAccept()
        }
      }
    }
  }

  ConfirmDialog {
    id: overwriteDlg
    parent: root
    anchors.fill: parent
    z: 100
    opened: root.overwritePath.length > 0
    message: "A file named “" + String(root.overwritePath).split("/").pop() + "” already exists. Replace it?"
    cancelText: "Cancel"
    confirmText: "Replace"
    onCanceled: root.overwritePath = ""
    onConfirmed: {
      var path = root.overwritePath
      root.overwritePath = ""
      root.emitPicked([path])
    }
  }

  QQC2.Popup {
    id: ellipsisPopup
    property var anchorItem: null
    parent: QQC2.Overlay.overlay
    padding: Style.spacing.hairline
    focus: true
    clip: true
    width: Style.space(220)
    implicitHeight: Math.min(
      Math.max(1, root.ellipsisMenuItems.length) * root.rowHeight
        + Math.max(0, root.ellipsisMenuItems.length - 1) * Style.spacing.labelGap
        + Style.spacing.xxs,
      root.rowHeight * 8 + 7 * Style.spacing.labelGap + Style.spacing.xxs)

    background: BorderSurface {
      color: Color.popups.background
      borderSpec: Border.localOrSurfaceSpec("popups", "border", Color.popups.border, Color.popups.border, Style.normalBorderWidth)
      radius: Style.cornerRadius
    }

    onAboutToShow: {
      var overlay = QQC2.Overlay.overlay
      var gap = Style.spacing.xxs
      var anchor = ellipsisPopup.anchorItem
      if (!overlay || !anchor) {
        ellipsisPopup.parent = anchor || root
        ellipsisPopup.x = 0
        ellipsisPopup.y = (anchor ? anchor.height : 0) + gap
        return
      }
      var pos = anchor.mapToItem(overlay, 0, 0)
      ellipsisPopup.x = pos.x
      ellipsisPopup.y = pos.y + anchor.height + gap
      var maxY = overlay.height - ellipsisPopup.implicitHeight - gap
      if (ellipsisPopup.y > maxY)
        ellipsisPopup.y = Math.max(gap, pos.y - ellipsisPopup.implicitHeight - gap)
    }

    contentItem: ListView {
      id: ellipsisList
      clip: true
      spacing: Style.spacing.labelGap
      boundsBehavior: Flickable.StopAtBounds
      model: root.ellipsisMenuItems
      implicitHeight: contentHeight

      delegate: Rectangle {
        required property var modelData
        required property int index
        width: ellipsisList.width
        height: root.rowHeight
        color: ellipsisHover.hovered
          ? Style.hoverFillFor(root.contentText, Color.accent)
          : "transparent"
        radius: Style.cornerRadius

        Text {
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.verticalCenter: parent.verticalCenter
          anchors.leftMargin: Style.spacing.controlPaddingX
          anchors.rightMargin: Style.spacing.controlPaddingX
          text: String(modelData.label)
          color: root.contentText
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          elide: Text.ElideMiddle
        }

        HoverHandler { id: ellipsisHover }

        MouseArea {
          anchors.fill: parent
          cursorShape: Qt.PointingHandCursor
          onClicked: {
            root.goTo(modelData.path)
            ellipsisPopup.close()
          }
        }
      }
    }
  }

  Item {
    id: newFolderOverlay
    parent: root
    anchors.fill: parent
    z: 100
    visible: root.newFolderOpen

    Rectangle {
      anchors.fill: parent
      color: Util.alpha(Color.popups.background, 0.82)
      MouseArea { anchors.fill: parent; onClicked: root.newFolderOpen = false }
    }

    BorderSurface {
      id: newFolderCard
      width: Math.min(parent.width - Style.space(32), Style.space(380))
      height: Style.space(190)
      anchors.centerIn: parent
      color: Color.popups.background
      borderSpec: Border.localOrSurfaceSpec("popups", "border", Color.popups.border, Color.popups.border, Style.normalBorderWidth)
      padding: Style.space(18)
      radius: Style.cornerRadius

      MouseArea { anchors.fill: parent; onClicked: {} }

      ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: newFolderCard.contentTopInset
        anchors.rightMargin: newFolderCard.contentRightInset
        anchors.bottomMargin: newFolderCard.contentBottomInset
        anchors.leftMargin: newFolderCard.contentLeftInset
        spacing: Style.space(10)

        Text {
          text: "New folder"
          color: root.contentText
          font.family: Style.font.family
          font.pixelSize: Style.font.title
        }
        TextField {
          Layout.fillWidth: true
          placeholderText: "Folder name"
          text: root.newFolderName
          onTextChanged: root.newFolderName = text
          onAccepted: root.createFolder()
        }
        Text {
          visible: root.newFolderError.length > 0
          text: root.newFolderError
          color: Color.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
        Row {
          Layout.alignment: Qt.AlignRight
          spacing: Style.space(8)
          Button { text: "Cancel"; bordered: true; onClicked: root.newFolderOpen = false }
          Button { text: "Create"; bordered: true; selected: true; onClicked: root.createFolder() }
        }
      }
    }
  }
}
