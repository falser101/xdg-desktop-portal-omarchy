import QtQuick
import QtQuick.Layouts
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

  title: String(request.title || (saveMode ? "Save" : (dirMode ? "Select folder" : "Open")))
  acceptText: String(request.accept_label || (saveMode || saveFilesMode ? "Save" : (dirMode ? "Select" : "Open")))
  acceptable: canAccept()
  showButtons: false
  cardWidth: Style.space(960)
  cardHeight: Style.space(620)
  focus: true
  readonly property int footerBarHeight: Style.spacing.controlHeight

  signal picked(var paths, var choices, var currentFilter)

  readonly property var places: extra.places || [
    { label: "Home", path: Quickshell.env("HOME") || "/" }
  ]
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

  function goTo(path) {
    var p = String(path || "")
    if (!p) return
    root.folder = folderUrl(p)
    root.selectedPaths = []
    root.selectedIsDir = false
    root.previewText = ""
  }

  function goParent() {
    var path = currentPath.replace(/\/$/, "")
    if (path === "" || path === "/") return
    var parent = path.replace(/\/[^\/]+$/, "")
    goTo(parent.length ? parent : "/")
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
      emitPicked(selectedPaths.length && selectedIsDir ? selectedPaths.slice() : [currentPath])
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

  function fileIcon(isDir, name) {
    if (isDir) return Quickshell.iconPath("folder", true)
    if (isImagePath(name)) return Quickshell.iconPath("image-x-generic", true)
    return Quickshell.iconPath("text-x-generic", true)
  }

  function refreshList() {
    var f = files.folder
    files.folder = "file:///proc/self"
    files.folder = f
    refreshNames()
  }

  function refreshNames() {
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

  onAccepted: tryAccept()
  onFolderChanged: refreshNames()
  Component.onCompleted: {
    refreshNames()
    var initial = {}
    for (var i = 0; i < choices.length; i++)
      initial[String(choices[i].id || i)] = String(choices[i].selected || (choices[i].options && choices[i].options.length ? choices[i].options[0][0] : "false"))
    choiceValues = initial
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
    showFiles: !root.dirMode || root.saveMode
    showDotAndDotDot: false
    showHidden: root.showHidden
    sortField: FolderListModel.Name
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
    spacing: Style.space(8)

    RowLayout {
      Layout.fillWidth: true
      Layout.fillHeight: true
      spacing: Style.space(10)

      ListView {
        id: placeList
        Layout.preferredWidth: Style.space(168)
        Layout.fillHeight: true
        clip: true
        model: root.places
        delegate: Item {
          required property var modelData
          width: placeList.width
          height: Style.space(30)
          readonly property bool sel: root.currentPath === String(modelData.path).replace(/\/$/, "") || root.currentPath === String(modelData.path)
          Rectangle {
            anchors.fill: parent
            color: sel ? Style.selectedFillFor(Color.foreground, Color.accent) : "transparent"
            radius: Style.cornerRadius
            Text {
              anchors.verticalCenter: parent.verticalCenter
              anchors.left: parent.left
              anchors.leftMargin: Style.space(8)
              anchors.right: parent.right
              text: String(modelData.label || modelData.path)
              color: sel ? Color.accent : Color.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              elide: Text.ElideRight
            }
          }
          MouseArea {
            anchors.fill: parent
            onClicked: root.goTo(modelData.path)
          }
        }
      }

      ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: Style.space(8)

        RowLayout {
          Layout.fillWidth: true
          spacing: Style.space(8)
          Button {
            text: "↑"
            bordered: true
            onClicked: root.goParent()
          }
          TextField {
            Layout.fillWidth: true
            text: root.currentPath
            onAccepted: {
              var p = text
              if (p.charAt(0) !== "/") p = "/" + p
              root.goTo(p)
            }
          }
          TextField {
            Layout.preferredWidth: Style.space(180)
            placeholderText: "Search"
            onTextChanged: root.query = text
          }
          Button {
            text: "New folder"
            bordered: true
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
            onClicked: root.showHidden = !root.showHidden
          }
        }

        RowLayout {
          Layout.fillWidth: true
          Layout.fillHeight: true
          spacing: Style.space(8)

          ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: files
            delegate: Item {
              width: list.width
              height: root.rowVisible(fileName, fileIsDir) ? Style.space(34) : 0
              visible: height > 0
              property string fileName: model.fileName || ""
              property bool fileIsDir: model.fileIsDir === true
              property var fileSize: model.fileSize || 0
              readonly property string path: String(model.filePath || model.fileURL || "").replace(/^file:\/\//, "")
              readonly property bool sel: root.selectedPaths.indexOf(path) !== -1

              Rectangle {
                anchors.fill: parent
                color: sel ? Style.selectedFillFor(Color.foreground, Color.accent) : "transparent"
                radius: Style.cornerRadius

                Image {
                  id: rowIcon
                  anchors.verticalCenter: parent.verticalCenter
                  anchors.left: parent.left
                  anchors.leftMargin: Style.space(8)
                  width: Style.space(18)
                  height: Style.space(18)
                  fillMode: Image.PreserveAspectFit
                  source: root.fileIcon(fileIsDir, fileName)
                  asynchronous: true
                }

                Text {
                  anchors.verticalCenter: parent.verticalCenter
                  anchors.left: rowIcon.right
                  anchors.leftMargin: Style.space(8)
                  anchors.right: sizeLabel.left
                  anchors.rightMargin: Style.space(8)
                  text: fileName
                  color: sel ? Color.accent : Color.foreground
                  font.family: Style.font.family
                  font.pixelSize: Style.font.body
                  elide: Text.ElideRight
                }

                Text {
                  id: sizeLabel
                  visible: !fileIsDir
                  anchors.verticalCenter: parent.verticalCenter
                  anchors.right: parent.right
                  anchors.rightMargin: Style.space(10)
                  text: root.formatSize(fileSize)
                  color: Color.muted
                  font.family: Style.font.family
                  font.pixelSize: Style.font.caption
                }
              }

              MouseArea {
                anchors.fill: parent
                onClicked: root.toggleSelect(path, fileIsDir, fileName)
                onDoubleClicked: {
                  if (fileIsDir) {
                    root.goTo(path)
                  } else if (root.saveMode) {
                    root.filename = fileName
                    root.tryAccept()
                  } else {
                    root.selectedPaths = [path]
                    root.tryAccept()
                  }
                }
              }
            }
          }

          Rectangle {
            visible: root.previewVisible
            Layout.preferredWidth: Style.space(220)
            Layout.fillHeight: true
            color: "transparent"
            border.color: Color.muted
            border.width: 1
            radius: Style.cornerRadius

            Column {
              anchors.fill: parent
              anchors.margins: Style.space(10)
              spacing: Style.space(8)

              Text {
                width: parent.width
                text: root.previewPath.length ? root.previewPath.split("/").pop() : "Preview"
                color: Color.foreground
                font.family: Style.font.family
                font.pixelSize: Style.font.body
                elide: Text.ElideMiddle
              }

              Image {
                width: parent.width
                height: Math.min(Style.space(160), parent.height - Style.space(80))
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
                color: Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
                wrapMode: Text.WrapAnywhere
                maximumLineCount: 14
                elide: Text.ElideRight
              }

              Text {
                width: parent.width
                visible: !root.previewIsImage && !root.previewIsText
                text: !root.previewPath.length ? "Select a file to preview" : (root.previewIsDir ? "Folder" : "No preview")
                color: Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.body
                wrapMode: Text.WordWrap
              }
            }
          }
        }
      }
    }

    RowLayout {
      Layout.fillWidth: true
      Layout.preferredHeight: root.footerBarHeight
      Layout.minimumHeight: root.footerBarHeight
      Layout.maximumHeight: root.footerBarHeight
      spacing: Style.space(8)

      TextField {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: root.saveMode
        verticalPadding: Style.spacing.controlPaddingY
        placeholderText: "File name"
        text: root.filename
        onTextChanged: root.filename = text
        onAccepted: root.tryAccept()
      }

      UpDropdown {
        visible: root.filters.length > 0
        Layout.preferredWidth: Style.space(220)
        Layout.fillHeight: true
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

      Item {
        Layout.fillWidth: true
        visible: !root.saveMode
      }

      Button {
        Layout.fillHeight: true
        Layout.preferredHeight: root.footerBarHeight
        text: root.cancelText
        bordered: true
        selected: root.selectedIndex === 0
        onClicked: {
          root.selectedIndex = 0
          root.rejected()
        }
      }
      Button {
        Layout.fillHeight: true
        Layout.preferredHeight: root.footerBarHeight
        text: root.acceptText
        bordered: true
        selected: root.selectedIndex === 1
        enabled: root.acceptable
        onClicked: {
          root.selectedIndex = 1
          if (root.acceptable) root.accepted()
        }
      }
    }

    Flow {
      Layout.fillWidth: true
      spacing: Style.space(8)
      visible: root.choices.length > 0
      Repeater {
        model: root.choices
        Toggle {
          required property var modelData
          label: String(modelData.label || modelData.id)
          checked: String(root.choiceValues[String(modelData.id)] || modelData.selected) === "true"
          visible: !(modelData.options && modelData.options.length)
          onClicked: {
            var next = Object.assign({}, root.choiceValues)
            next[String(modelData.id)] = checked ? "false" : "true"
            root.choiceValues = next
          }
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

  Item {
    id: newFolderOverlay
    parent: root
    anchors.fill: parent
    z: 100
    visible: root.newFolderOpen

    Rectangle {
      anchors.fill: parent
      color: Util.alpha(Color.background, 0.7)
      MouseArea { anchors.fill: parent; onClicked: root.newFolderOpen = false }
    }

    BorderSurface {
      id: newFolderCard
      width: Math.min(parent.width - Style.space(32), Style.space(380))
      height: Style.space(190)
      anchors.centerIn: parent
      color: Color.background
      borderSpec: Border.flat(Color.accent, Style.normalBorderWidth)
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
          color: Color.foreground
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
