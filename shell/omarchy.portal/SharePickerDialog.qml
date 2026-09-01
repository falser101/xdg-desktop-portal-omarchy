import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import qs.Commons
import qs.Ui

// Omarchy share picker. Same capture path as the window-preview plugin:
// ScreencopyView + ShellScreen (displays) / Toplevel.wayland (windows).
// Pages: Display | Windows | Region.
PortalDialog {
  id: root

  property var extra: ({})
  property string query: ""
  property string selected: ""
  property var selectedGeometries: []
  property bool allowToken: extra.allowToken === true
  property string page: "display"

  readonly property var allScreens: extra.screens || []
  readonly property var allWindows: extra.windows || []
  readonly property var hyprToplevels: (Hyprland.toplevels && Hyprland.toplevels.values)
    ? Hyprland.toplevels.values
    : []
  readonly property int shellScreenCount: Quickshell.screens ? Quickshell.screens.length : 0

  title: "Screen Sharing Requested"
  subtitle: "Choose a display, window, or region to share"
  acceptText: "Share"
  showButtons: true
  acceptable: selected.length > 0
  cardWidth: Style.space(920)
  cardHeight: Style.space(720)
  focus: true

  signal picked(string line, bool allowToken)

  readonly property var realScreens: {
    var out = []
    for (var i = 0; i < allScreens.length; i++) {
      if (!allScreens[i].synthetic)
        out.push(allScreens[i])
    }
    return out
  }

  readonly property var filteredWindows: {
    var items = allWindows
    var geos = selectedGeometries
    var q = query.toLowerCase()
    var out = []
    for (var i = 0; i < items.length; i++) {
      var w = items[i]
      if (geos.length > 0 && !intersectsAny(w, geos))
        continue
      if (q.length > 0) {
        var hay = (String(w.label || "") + " " + String(w.className || "") + " " + String(w.title || "")).toLowerCase()
        if (hay.indexOf(q) === -1)
          continue
      }
      out.push(w)
    }
    return out
  }

  readonly property var filteredScreenValues: {
    var _n = root.shellScreenCount
    var screens = Quickshell.screens || []
    var out = []
    for (var i = 0; i < screens.length; i++) {
      var s = screens[i]
      if (!root.screenMatches(s))
        continue
      var extra = root.extraForScreen(s)
      out.push(String(extra.value || ("screen:" + (s.name || ""))))
    }
    return out
  }

  readonly property int filteredScreenCount: (filteredScreenValues || []).length

  readonly property var selectableValues: {
    if (page === "region")
      return ["REGION_PICK"]
    if (page === "windows") {
      var wins = filteredWindows || []
      var wout = []
      for (var i = 0; i < wins.length; i++)
        wout.push(String(wins[i].value || ""))
      return wout
    }
    return filteredScreenValues || []
  }

  readonly property int minCardWidth: Style.space(260)
  readonly property int gridGap: Style.space(10)
  readonly property int gridColumns: {
    var w = scroller && scroller.width > 0 ? scroller.width : (cardWidth - Style.space(40))
    var cols = Math.floor((w + gridGap) / (minCardWidth + gridGap))
    if (cols < 1)
      cols = 1
    if (cols > 6)
      cols = 6
    return cols
  }
  readonly property int tileCardHeight: gridColumns >= 4 ? Style.space(180) : (gridColumns >= 3 ? Style.space(200) : Style.space(220))
  readonly property real tileCardWidth: {
    var cols = Math.max(1, gridColumns)
    var w = scroller && scroller.width > 0 ? scroller.width : (cardWidth - Style.space(40))
    return Math.max(Style.space(160), (w - gridGap * (cols - 1)) / cols)
  }

  readonly property real displayTileWidth: {
    var n = Math.max(1, Math.min(2, filteredScreenCount))
    var w = scroller && scroller.width > 0 ? scroller.width : (cardWidth - Style.space(40))
    if (filteredScreenCount <= 1)
      return Math.min(w, Style.space(680))
    return (w - gridGap) / n
  }

  function chunkValueRows(items, cols) {
    var rows = []
    var list = items || []
    var n = Math.max(1, cols || 1)
    for (var i = 0; i < list.length; i += n) {
      var row = []
      for (var j = 0; j < n && i + j < list.length; j++)
        row.push(String(list[i + j].value || list[i + j] || ""))
      rows.push(row)
    }
    return rows
  }

  readonly property var selectionRows: {
    if (page === "region")
      return [["REGION_PICK"]]
    if (page === "windows")
      return chunkValueRows(filteredWindows || [], gridColumns)
    return chunkValueRows(filteredScreenValues || [], Math.max(1, Math.min(2, filteredScreenCount)))
  }

  readonly property bool scrollerOverflow: scroller ? (scroller.contentHeight > scroller.height + 1) : false

  function extraForScreen(screen) {
    var name = screen ? String(screen.name || "") : ""
    var extras = root.realScreens || []
    for (var i = 0; i < extras.length; i++) {
      if (String(extras[i].name || "") === name)
        return extras[i]
    }
    var model = screen ? String(screen.model || "") : ""
    var w = screen ? Number(screen.width || 0) : 0
    var h = screen ? Number(screen.height || 0) : 0
    var dpr = screen && screen.devicePixelRatio ? Number(screen.devicePixelRatio) : 0
    if (dpr > 1) {
      w = Math.round(w / dpr)
      h = Math.round(h / dpr)
    }
    return {
      "name": name,
      "label": model && model !== name ? (model + " (" + name + ")") : name,
      "value": "screen:" + name,
      "width": w,
      "height": h,
      "x": screen ? Number(screen.x || 0) : 0,
      "y": screen ? Number(screen.y || 0) : 0,
      "tasks": []
    }
  }

  function screenMatches(screen) {
    if (!screen)
      return false
    var extra = extraForScreen(screen)
    var q = query.toLowerCase()
    if (q.length) {
      var hay = (String(extra.label || "") + " " + String(extra.name || "") + " " + String(screen.name || "") + " " + String(screen.model || "")).toLowerCase()
      if (hay.indexOf(q) === -1)
        return false
    }
    var geos = selectedGeometries
    if (geos.length > 0 && !intersectsAny(extra, geos))
      return false
    return true
  }

  function screenResolution(screen, extra) {
    var w = Number((extra && extra.width) || (screen && screen.width) || 0)
    var h = Number((extra && extra.height) || (screen && screen.height) || 0)
    if (w > 0 && h > 0)
      return Math.round(w) + " \u00d7 " + Math.round(h)
    return ""
  }

  function intersectsAny(item, geos) {
    var ax = Number(item.x || 0), ay = Number(item.y || 0)
    var aw = Number(item.width || 0), ah = Number(item.height || 0)
    for (var i = 0; i < geos.length; i++) {
      var g = geos[i]
      if (ax < g.x + g.width && ax + aw > g.x && ay < g.y + g.height && ay + ah > g.y)
        return true
    }
    return false
  }

  function geometryOf(item) {
    return {
      "x": Number(item.x || 0),
      "y": Number(item.y || 0),
      "width": Number(item.width || 0),
      "height": Number(item.height || 0),
      "name": String(item.name || "")
    }
  }

  function chipChecked(name) {
    for (var i = 0; i < selectedGeometries.length; i++) {
      if (selectedGeometries[i].name === name)
        return true
    }
    return false
  }

  function toggleChip(item) {
    var name = String(item.name || "")
    var next = []
    var found = false
    for (var i = 0; i < selectedGeometries.length; i++) {
      if (selectedGeometries[i].name === name) {
        found = true
        continue
      }
      next.push(selectedGeometries[i])
    }
    if (!found)
      next.push(geometryOf(item))
    selectedGeometries = next
  }

  function normalizeAddr(addr) {
    var a = String(addr || "").trim().toLowerCase()
    if (!a || a === "0" || a === "null" || a === "undefined")
      return ""
    if (a.indexOf("0x") === 0)
      return a
    if (/^\d+$/.test(a)) {
      try {
        return "0x" + BigInt(a).toString(16)
      } catch (e) {
        var n = parseInt(a, 10)
        if (!isFinite(n))
          return a
        return "0x" + n.toString(16)
      }
    }
    return a
  }

  function toplevelWaylandForItem(item) {
    if (!item)
      return null
    var tops = root.hyprToplevels
    var _n = tops.length
    var want = normalizeAddr(item.address || item.handle || "")
    if (want) {
      for (var i = 0; i < tops.length; i++) {
        var t = tops[i]
        var ipcAddr = t.lastIpcObject ? t.lastIpcObject.address : ""
        var a = normalizeAddr(t.address || ipcAddr || "")
        if (a && a === want)
          return t.wayland || null
      }
    }
    var klass = String(item.className || item.class || "")
    var title = String(item.title || "")
    var classHits = []
    for (var j = 0; j < tops.length; j++) {
      var top = tops[j]
      if (!top.wayland)
        continue
      var ipc = top.lastIpcObject || {}
      var tc = String(ipc.class || ipc.initialClass || "")
      var tt = String(top.title || ipc.title || "")
      if (klass && tc === klass && title && tt === title)
        return top.wayland
      if (klass && tc === klass)
        classHits.push(top.wayland)
    }
    if (classHits.length === 1)
      return classHits[0]
    return null
  }

  function iconSource(icon) {
    var value = String(icon || "")
    if (value.indexOf("file://") === 0 || value.indexOf("image://") === 0)
      return value
    if (value.charAt(0) === "/")
      return "file://" + value
    if (!value.length)
      return Quickshell.iconPath("application-x-executable", true)
    var themed = Quickshell.iconPath(value, true)
    if (themed && themed.length)
      return themed
    return Quickshell.iconPath("application-x-executable", true)
  }

  function isSelected(value) {
    return selected.length > 0 && selected === String(value || "")
  }

  function selectValue(value) {
    selected = String(value || "")
    Qt.callLater(ensureSelectedVisible)
  }

  function setPage(id) {
    var next = String(id || "display")
    if (page === next) {
      Qt.callLater(ensureDefaultSelection)
      return
    }
    page = next
    if (scroller)
      scroller.contentY = 0
    Qt.callLater(ensureDefaultSelection)
  }

  function ensureDefaultSelection() {
    var values = selectableValues
    if (!values || values.length === undefined) {
      selected = ""
      return
    }
    if (values.length === 0) {
      selected = page === "region" ? "REGION_PICK" : ""
      return
    }
    for (var i = 0; i < values.length; i++) {
      if (values[i] === selected)
        return
    }
    selected = values[0]
  }

  function findSelectedCell() {
    var rows = selectionRows || []
    for (var r = 0; r < rows.length; r++) {
      var row = rows[r] || []
      for (var c = 0; c < row.length; c++) {
        if (row[c] === selected)
          return { "r": r, "c": c }
      }
    }
    return { "r": 0, "c": 0 }
  }

  function moveSelection(dx, dy) {
    var rows = selectionRows || []
    if (!rows.length)
      return
    var cell = findSelectedCell()
    var r = cell.r
    var c = cell.c
    if (r >= rows.length)
      r = 0
    if (dx !== 0) {
      c += dx
      if (c < 0) {
        r = Math.max(0, r - 1)
        c = Math.max(0, (rows[r] || []).length - 1)
      } else if (c >= (rows[r] || []).length) {
        if (r + 1 < rows.length) {
          r += 1
          c = 0
        } else {
          c = Math.max(0, (rows[r] || []).length - 1)
        }
      }
    }
    if (dy !== 0) {
      r = Math.max(0, Math.min(rows.length - 1, r + dy))
      c = Math.min(c, Math.max(0, (rows[r] || []).length - 1))
    }
    var pick = (rows[r] || [])[c]
    if (pick !== undefined && pick !== null)
      selectValue(pick)
  }

  function confirmSelection() {
    if (!selected.length)
      return false
    if (selected === "REGION_PICK") {
      picked("REGION_PICK", allowToken)
      return true
    }
    picked((allowToken ? "[SELECTION]r/" : "[SELECTION]/") + selected, allowToken)
    return true
  }

  function ensureSelectedVisible() {
    if (!scrollerOverflow)
      return
    var values = selectableValues
    var idx = -1
    for (var i = 0; i < values.length; i++) {
      if (values[i] === selected) {
        idx = i
        break
      }
    }
    if (idx < 0 || values.length <= 1)
      return
    var ratio = idx / Math.max(1, values.length - 1)
    var maxY = Math.max(0, scroller.contentHeight - scroller.height)
    scroller.contentY = ratio * maxY
  }

  function handleNavKey(e) {
    if (e.key === Qt.Key_Left) {
      moveSelection(-1, 0)
      return true
    }
    if (e.key === Qt.Key_Right) {
      moveSelection(1, 0)
      return true
    }
    if (e.key === Qt.Key_Up) {
      moveSelection(0, -1)
      return true
    }
    if (e.key === Qt.Key_Down) {
      moveSelection(0, 1)
      return true
    }
    if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) {
      if (confirmSelection())
        return true
    }
    if (e.key === Qt.Key_Escape) {
      rejected()
      return true
    }
    return false
  }

  onAccepted: confirmSelection()
  onQueryChanged: Qt.callLater(ensureDefaultSelection)
  onSelectedGeometriesChanged: Qt.callLater(ensureDefaultSelection)
  onAllScreensChanged: Qt.callLater(ensureDefaultSelection)
  onAllWindowsChanged: Qt.callLater(ensureDefaultSelection)
  onPageChanged: Qt.callLater(ensureDefaultSelection)
  onShellScreenCountChanged: Qt.callLater(ensureDefaultSelection)
  Component.onCompleted: {
    try { Hyprland.refreshToplevels() } catch (e) {}
    Qt.callLater(ensureDefaultSelection)
  }

  Keys.onPressed: function (e) {
    if (handleNavKey(e)) {
      e.accepted = true
      return
    }
    if (e.key === Qt.Key_Tab || e.key === Qt.Key_Backtab) {
      if (handleKey(e))
        e.accepted = true
    }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(10)

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.space(8)

      Repeater {
        model: [
          { "id": "display", "label": "Display" },
          { "id": "windows", "label": "Windows" },
          { "id": "region", "label": "Region" }
        ]
        delegate: Button {
          required property var modelData
          text: modelData.label
          selected: root.page === modelData.id
          bordered: true
          Layout.alignment: Qt.AlignVCenter
          onClicked: root.setPage(modelData.id)
        }
      }

      Item {
        Layout.fillWidth: true
      }

      Flow {
        Layout.fillWidth: false
        spacing: Style.space(6)
        visible: root.page === "windows" && root.realScreens.length > 1

        Repeater {
          model: root.realScreens
          delegate: Rectangle {
            required property var modelData
            readonly property bool on: root.chipChecked(String(modelData.name || ""))
            height: Style.space(28)
            width: chipLabel.implicitWidth + Style.space(20)
            radius: height / 2
            color: on ? Color.menu.selectedBackground : Util.alpha(Color.popups.text, 0.10)
            border.width: 1
            border.color: on ? Color.menu.selectedBorder : Util.alpha(Color.popups.text, 0.22)

            Text {
              id: chipLabel
              anchors.centerIn: parent
              text: String(modelData.name || modelData.label || "")
              color: on ? Color.menu.selectedText : Color.popups.text
              font.family: Style.font.family
              font.pixelSize: Style.font.caption || Style.font.body
            }

            MouseArea {
              anchors.fill: parent
              cursorShape: Qt.PointingHandCursor
              onClicked: root.toggleChip(modelData)
            }
          }
        }
      }

      TextField {
        id: searchField
        visible: root.page !== "region"
        Layout.preferredWidth: Style.space(220)
        Layout.alignment: Qt.AlignRight
        placeholderText: root.page === "windows" ? "Search windows…" : "Search displays…"
        text: root.query
        onTextChanged: root.query = text
        Keys.onPressed: function (e) {
          if (root.handleNavKey(e))
            e.accepted = true
        }
      }
    }

    RowLayout {
      Layout.fillWidth: true
      Layout.fillHeight: true
      spacing: Style.space(6)

      Flickable {
        id: scroller
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.VerticalFlick
        interactive: true
        contentWidth: width
        contentHeight: Math.max(height, contentCol.implicitHeight)
        maximumFlickVelocity: 2500

        WheelHandler {
          acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
          onWheel: function (event) {
            var step = event.angleDelta.y
            if (step === 0)
              return
            var next = scroller.contentY - step
            var maxY = Math.max(0, scroller.contentHeight - scroller.height)
            scroller.contentY = Math.max(0, Math.min(maxY, next))
            event.accepted = true
          }
        }

        ColumnLayout {
          id: contentCol
          width: scroller.width
          spacing: Style.space(12)

          // —— Display — ShellScreen capture, same API as window-preview. ——
          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.space(10)
            visible: root.page === "display"

            Text {
              Layout.fillWidth: true
              text: "Share an entire display"
              color: Color.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.body
            }

            Flow {
              Layout.fillWidth: true
              spacing: root.gridGap

              Repeater {
                model: Quickshell.screens

                BorderSurface {
                  id: displayCard
                  required property var modelData
                  required property int index
                  readonly property var screen: modelData
                  readonly property var extra: root.extraForScreen(screen)
                  readonly property string value: String(extra.value || ("screen:" + (screen && screen.name || "")))
                  readonly property bool matches: root.screenMatches(screen)
                  readonly property bool sel: root.isSelected(value)
                  readonly property bool hot: displayHover.hovered || sel
                  readonly property string resText: root.screenResolution(screen, extra)
                  readonly property real aspectW: Math.max(1, Number(extra.width || screen.width || 16))
                  readonly property real aspectH: Math.max(1, Number(extra.height || screen.height || 9))

                  visible: matches
                  width: visible ? root.displayTileWidth : 0
                  height: visible ? (Math.round(root.displayTileWidth * aspectH / aspectW) + Style.space(72)) : 0

                  color: hot ? Color.menu.selectedBackground : Color.popups.background
                  borderSpec: sel || hot
                    ? Border.surfaceSpec("menu", "selected-border", Color.menu.selectedBorder, Style.normalBorderWidth)
                    : Border.flat(Util.alpha(Color.popups.text, 0.18), 1)
                  radius: Style.cornerRadius
                  padding: Style.space(10)
                  clip: true

                  ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Style.space(8)
                    spacing: Style.space(8)

                    RowLayout {
                      Layout.fillWidth: true
                      spacing: Style.space(8)

                      Rectangle {
                        Layout.preferredHeight: Style.space(22)
                        Layout.preferredWidth: badgeLabel.implicitWidth + Style.space(12)
                        radius: height / 2
                        color: Util.alpha(Color.popups.text, 0.12)

                        Text {
                          id: badgeLabel
                          anchors.centerIn: parent
                          text: "Display"
                          color: displayCard.hot ? Color.menu.selectedText : Color.popups.text
                          font.family: Style.font.family
                          font.pixelSize: Style.font.caption || Style.font.body
                          font.bold: true
                        }
                      }

                      Text {
                        Layout.fillWidth: true
                        text: String(displayCard.screen && displayCard.screen.name || displayCard.extra.name || "Display")
                        color: displayCard.hot ? Color.menu.selectedText : Color.popups.text
                        font.family: Style.font.family
                        font.pixelSize: Style.font.body
                        font.bold: true
                        elide: Text.ElideRight
                      }

                      Text {
                        visible: displayCard.resText.length > 0
                        text: displayCard.resText
                        color: Color.muted
                        font.family: Style.font.family
                        font.pixelSize: Style.font.caption || Style.font.body
                      }
                    }

                    Item {
                      id: previewHost
                      Layout.fillWidth: true
                      Layout.fillHeight: true
                      clip: true

                      Rectangle {
                        anchors.fill: parent
                        color: Util.alpha(Color.popups.text, 0.06)
                        radius: Style.cornerRadius
                      }

                      Column {
                        anchors.centerIn: parent
                        spacing: Style.space(4)
                        visible: !displayLive.hasContent
                        width: parent.width - Style.space(16)

                        Text {
                          width: parent.width
                          horizontalAlignment: Text.AlignHCenter
                          text: "Entire display"
                          color: Color.popups.text
                          font.family: Style.font.family
                          font.pixelSize: Style.font.body
                          font.bold: true
                        }
                        Text {
                          width: parent.width
                          horizontalAlignment: Text.AlignHCenter
                          text: displayCard.resText.length ? displayCard.resText : String(displayCard.screen && displayCard.screen.name || "")
                          color: Color.muted
                          font.family: Style.font.family
                          font.pixelSize: Style.font.caption || Style.font.body
                        }
                      }

                      ScreencopyView {
                        id: displayLive
                        readonly property real srcW: sourceSize.width > 0 ? sourceSize.width : displayCard.aspectW
                        readonly property real srcH: sourceSize.height > 0 ? sourceSize.height : displayCard.aspectH
                        readonly property real fit: Math.min(
                          previewHost.width / Math.max(1, srcW),
                          previewHost.height / Math.max(1, srcH)
                        )
                        width: Math.max(1, srcW * fit)
                        height: Math.max(1, srcH * fit)
                        anchors.centerIn: parent
                        captureSource: displayCard.screen
                        live: false
                        paintCursor: false
                        visible: hasContent
                        constraintSize: Qt.size(previewHost.width, previewHost.height)

                        function recapture() {
                          if (!captureSource)
                            return
                          captureFrame()
                        }

                        onCaptureSourceChanged: Qt.callLater(recapture)
                        onVisibleChanged: if (visible && captureSource && !hasContent) Qt.callLater(recapture)
                        Component.onCompleted: Qt.callLater(recapture)
                      }

                      Timer {
                        interval: 180
                        repeat: true
                        running: displayCard.matches && !!displayLive.captureSource && !displayLive.hasContent
                        onTriggered: displayLive.recapture()
                      }
                    }

                    Text {
                      Layout.fillWidth: true
                      text: String(displayCard.extra.label || displayCard.screen.model || "Entire display")
                      color: Color.muted
                      font.family: Style.font.family
                      font.pixelSize: Style.font.caption || Style.font.body
                      elide: Text.ElideRight
                    }
                  }

                  HoverHandler {
                    id: displayHover
                    cursorShape: Qt.PointingHandCursor
                  }
                  TapHandler {
                    onTapped: root.selectValue(displayCard.value)
                    onDoubleTapped: {
                      root.selectValue(displayCard.value)
                      root.confirmSelection()
                    }
                  }
                }
              }
            }

            Text {
              visible: root.filteredScreenCount === 0
              Layout.fillWidth: true
              horizontalAlignment: Text.AlignHCenter
              text: root.query.length ? "No matching displays" : "No displays found"
              color: Color.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.body
            }
          }

          // —— Windows — toplevel export via Hyprland.toplevels. ——
          GridLayout {
            Layout.fillWidth: true
            columns: root.gridColumns
            columnSpacing: root.gridGap
            rowSpacing: root.gridGap
            visible: root.page === "windows" && root.filteredWindows.length > 0

            Repeater {
              model: root.filteredWindows

              BorderSurface {
                id: windowCard
                required property var modelData
                readonly property var item: modelData
                readonly property bool sel: root.isSelected(item.value)
                readonly property bool hot: winHover.hovered || sel
                readonly property var captureSrc: {
                  var _dep = root.hyprToplevels.length
                  return root.toplevelWaylandForItem(item)
                }

                Layout.fillWidth: false
                Layout.preferredWidth: root.tileCardWidth
                Layout.maximumWidth: root.tileCardWidth
                Layout.preferredHeight: root.tileCardHeight
                color: hot ? Color.menu.selectedBackground : Color.popups.background
                borderSpec: sel || hot
                  ? Border.surfaceSpec("menu", "selected-border", Color.menu.selectedBorder, Style.normalBorderWidth)
                  : Border.flat(Util.alpha(Color.popups.text, 0.18), 1)
                radius: Style.cornerRadius
                padding: Style.space(10)

                ColumnLayout {
                  anchors.fill: parent
                  anchors.margins: Style.space(8)
                  spacing: Style.space(8)

                  RowLayout {
                    Layout.fillWidth: true
                    spacing: Style.space(8)

                    Image {
                      Layout.preferredWidth: Style.space(18)
                      Layout.preferredHeight: Style.space(18)
                      source: root.iconSource(windowCard.item.icon || windowCard.item.className)
                      fillMode: Image.PreserveAspectFit
                      asynchronous: true
                    }

                    Text {
                      Layout.fillWidth: true
                      text: String(windowCard.item.label || windowCard.item.title || "Window")
                      color: windowCard.hot ? Color.menu.selectedText : Color.popups.text
                      font.family: Style.font.family
                      font.pixelSize: Style.font.body
                      font.bold: true
                      elide: Text.ElideRight
                    }
                  }

                  Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: Util.alpha(Color.popups.text, 0.06)
                    radius: Style.cornerRadius
                    clip: true

                    ScreencopyView {
                      id: winLive
                      anchors.fill: parent
                      anchors.margins: Style.space(4)
                      captureSource: windowCard.captureSrc
                      live: !!windowCard.captureSrc
                      paintCursor: false
                      visible: !!windowCard.captureSrc && hasContent
                    }

                    Text {
                      anchors.centerIn: parent
                      visible: !winLive.visible
                      text: String(windowCard.item.className || windowCard.item.title || "Window")
                      color: Color.muted
                      font.family: Style.font.family
                      font.pixelSize: Style.font.caption || Style.font.body
                    }
                  }
                }

                HoverHandler {
                  id: winHover
                  cursorShape: Qt.PointingHandCursor
                }
                TapHandler {
                  onTapped: root.selectValue(String(windowCard.item.value || ""))
                  onDoubleTapped: {
                    root.selectValue(String(windowCard.item.value || ""))
                    root.confirmSelection()
                  }
                }
              }
            }
          }

          Text {
            visible: root.page === "windows" && root.filteredWindows.length === 0
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            text: root.query.length || root.selectedGeometries.length
              ? "No matching windows"
              : "No windows found"
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }

          // —— Region ——
          Item {
            Layout.fillWidth: true
            Layout.preferredHeight: Style.space(280)
            visible: root.page === "region"

            BorderSurface {
              id: regionCard
              anchors.centerIn: parent
              width: Math.min(parent.width, Style.space(420))
              height: Style.space(220)
              readonly property bool sel: root.isSelected("REGION_PICK")
              readonly property bool hot: regionHover.hovered || sel
              color: hot ? Color.menu.selectedBackground : Color.popups.background
              borderSpec: sel || hot
                ? Border.surfaceSpec("menu", "selected-border", Color.menu.selectedBorder, Style.normalBorderWidth)
                : Border.flat(Util.alpha(Color.popups.text, 0.18), 1)
              radius: Style.cornerRadius
              padding: Style.space(16)

              ColumnLayout {
                anchors.fill: parent
                anchors.margins: Style.space(12)
                spacing: Style.space(10)

                Rectangle {
                  Layout.preferredHeight: Style.space(22)
                  Layout.preferredWidth: regionBadge.implicitWidth + Style.space(12)
                  Layout.alignment: Qt.AlignHCenter
                  radius: height / 2
                  color: Util.alpha(Color.popups.text, 0.12)

                  Text {
                    id: regionBadge
                    anchors.centerIn: parent
                    text: "Region"
                    color: regionCard.hot ? Color.menu.selectedText : Color.popups.text
                    font.family: Style.font.family
                    font.pixelSize: Style.font.caption || Style.font.body
                    font.bold: true
                  }
                }

                Text {
                  Layout.fillWidth: true
                  horizontalAlignment: Text.AlignHCenter
                  text: "Share a region"
                  color: regionCard.hot ? Color.menu.selectedText : Color.popups.text
                  font.family: Style.font.family
                  font.pixelSize: Style.font.title
                  font.bold: true
                }

                Text {
                  Layout.fillWidth: true
                  Layout.fillHeight: true
                  horizontalAlignment: Text.AlignHCenter
                  wrapMode: Text.WordWrap
                  text: "The screen will freeze so you can drag a rectangle. Snap to a window or the whole display."
                  color: Color.muted
                  font.family: Style.font.family
                  font.pixelSize: Style.font.body
                }

                Button {
                  Layout.alignment: Qt.AlignHCenter
                  text: "Select region"
                  selected: true
                  onClicked: {
                    root.selectValue("REGION_PICK")
                    root.confirmSelection()
                  }
                }
              }

              HoverHandler {
                id: regionHover
                cursorShape: Qt.PointingHandCursor
              }
              TapHandler {
                onTapped: root.selectValue("REGION_PICK")
                onDoubleTapped: {
                  root.selectValue("REGION_PICK")
                  root.confirmSelection()
                }
              }
            }
          }
        }
      }

      Item {
        id: scrollBarSlot
        Layout.fillHeight: true
        Layout.preferredWidth: root.scrollerOverflow ? Style.space(10) : 0
        Layout.maximumWidth: Style.space(10)
        visible: root.scrollerOverflow

        QQC2.ScrollBar {
          id: scrollBar
          anchors.fill: parent
          orientation: Qt.Vertical
          policy: QQC2.ScrollBar.AlwaysOn
          size: Math.max(0.08, scroller.visibleArea.heightRatio)
          Binding on position {
            when: !scrollBar.pressed
            value: scroller.visibleArea.yPosition
          }
          onPositionChanged: {
            if (!pressed)
              return
            var range = Math.max(0, scroller.contentHeight - scroller.height)
            var maxPos = Math.max(0.0001, 1 - size)
            scroller.contentY = (position / maxPos) * range
          }
          contentItem: Rectangle {
            implicitWidth: Style.space(8)
            radius: width / 2
            color: scrollBar.pressed
              ? Color.menu.selectedBorder
              : (scrollBar.hovered ? Util.alpha(Color.popups.text, 0.50) : Util.alpha(Color.popups.text, 0.32))
          }
          background: Rectangle {
            implicitWidth: Style.space(10)
            radius: width / 2
            color: Util.alpha(Color.popups.text, 0.10)
          }
        }
      }
    }
  }

  footerLeft: Row {
    spacing: Style.space(8)

    Rectangle {
      width: Style.space(16)
      height: Style.space(16)
      anchors.verticalCenter: parent.verticalCenter
      radius: Style.space(3)
      color: root.allowToken ? Color.menu.selectedBackground : Util.alpha(Color.popups.text, 0.12)
      border.width: 1
      border.color: Util.alpha(Color.popups.text, root.allowToken ? 0.55 : 0.28)

      Text {
        anchors.centerIn: parent
        visible: root.allowToken
        text: "✓"
        color: Color.menu.selectedText
        font.pixelSize: Style.font.caption || Style.font.body
      }

      MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.allowToken = !root.allowToken
      }
    }

    Text {
      anchors.verticalCenter: parent.verticalCenter
      text: "Allow the application to do this without asking next time"
      color: Color.popups.text
      font.family: Style.font.family
      font.pixelSize: Style.font.body
      wrapMode: Text.NoWrap

      MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.allowToken = !root.allowToken
      }
    }
  }
}
