import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import qs.Commons
import qs.Ui

// Omarchy share-picker layout:
// header chips + search → Displays grid → "Windows" separator → window cards.
// Selection: default first item; arrows move; Enter / Share confirms.
// Previews: Quickshell ScreencopyView (live compositor capture), from the compositor.
// PipeWireSourceItem — dialog opens immediately; no grim/PNG prefetch.
PortalDialog {
  id: root

  property var extra: ({})
  property string query: ""
  property string selected: ""
  property var selectedGeometries: []
  property bool allowToken: extra.allowToken === true

  readonly property var allScreens: extra.screens || []
  readonly property var allWindows: extra.windows || []
  readonly property var hyprToplevels: (Hyprland.toplevels && Hyprland.toplevels.values)
    ? Hyprland.toplevels.values
    : []

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

  readonly property var syntheticScreens: {
    var out = []
    for (var i = 0; i < allScreens.length; i++) {
      if (allScreens[i].synthetic)
        out.push(allScreens[i])
    }
    return out
  }

  readonly property var filteredScreens: {
    var items = realScreens
    var geos = selectedGeometries
    var q = query.toLowerCase()
    var out = []
    for (var i = 0; i < items.length; i++) {
      var s = items[i]
      if (geos.length > 0 && !intersectsAny(s, geos))
        continue
      if (q.length > 0) {
        var hay = String(s.label || s.name || "").toLowerCase()
        if (hay.indexOf(q) === -1)
          continue
      }
      out.push(s)
    }
    return out
  }

  readonly property var filteredSynthetics: {
    var q = query.toLowerCase()
    if (!q.length)
      return syntheticScreens
    var out = []
    for (var i = 0; i < syntheticScreens.length; i++) {
      var s = syntheticScreens[i]
      var hay = String(s.label || s.name || "").toLowerCase()
      if (hay.indexOf(q) !== -1)
        out.push(s)
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

  // Flat list of selectable values in visual order (for ←→ / default).
  // Region is a footer action, not a grid card.
  readonly property var selectableValues: {
    var out = []
    var screens = filteredScreens || []
    var wins = filteredWindows || []
    var i
    for (i = 0; i < screens.length; i++)
      out.push(String(screens[i].value || ""))
    for (i = 0; i < wins.length; i++)
      out.push(String(wins[i].value || ""))
    return out
  }

  // Responsive columns from available scroller width (min card ~260px).
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
  // Tile size for Displays/Windows (not PortalDialog.cardHeight dialog chrome).
  readonly property int tileCardHeight: gridColumns >= 4 ? Style.space(180) : (gridColumns >= 3 ? Style.space(200) : Style.space(220))
  readonly property real tileCardWidth: {
    var cols = Math.max(1, gridColumns)
    var w = scroller && scroller.width > 0 ? scroller.width : (cardWidth - Style.space(40))
    return Math.max(Style.space(160), (w - gridGap * (cols - 1)) / cols)
  }

  function chunkValueRows(items, cols) {
    var rows = []
    var list = items || []
    var n = Math.max(1, cols || 1)
    for (var i = 0; i < list.length; i += n) {
      var row = []
      for (var j = 0; j < n && i + j < list.length; j++)
        row.push(String(list[i + j].value || ""))
      rows.push(row)
    }
    return rows
  }

  // Row-major grid — Displays and Windows share the same chunking.
  readonly property var selectionRows: {
    var cols = gridColumns
    return chunkValueRows(filteredScreens || [], cols).concat(chunkValueRows(filteredWindows || [], cols))
  }

  readonly property bool showWindowsSeparator: (filteredScreens || []).length > 0 && (filteredWindows || []).length > 0
  readonly property bool scrollerOverflow: scroller ? (scroller.contentHeight > scroller.height + 1) : false

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
    // Decimal from older payloads — prefer BigInt so >2^53 stays exact.
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

  function screenForName(name) {
    var n = String(name || "")
    if (!n.length)
      return null
    var screens = Quickshell.screens || []
    for (var i = 0; i < screens.length; i++) {
      if (String(screens[i].name || "") === n)
        return screens[i]
    }
    return null
  }

  function toplevelWaylandForItem(item) {
    if (!item)
      return null
    // Touch length so bindings refresh when Hyprland IPC updates.
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
    // Fallback: class + title (XDPH address is often "0" / unmapped).
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

  function field(item, key, fallback) {
    if (!item)
      return fallback || ""
    var value = item[key]
    if (value === undefined || value === null)
      return fallback || ""
    var text = String(value)
    return text === "undefined" ? (fallback || "") : text
  }

  function shareHeading(item) {
    var name = field(item, "name", "") || field(item, "label", "display")
    return "Share \u201C" + name + "\u201D"
  }

  function taskOverflowText(item) {
    if (!item)
      return ""
    var count = Number(item["taskCount"] || ((item["tasks"] || []).length) || 0)
    if (count === 0)
      return "No windows open"
    var overflow = Number(item["taskOverflow"] || 0)
    return overflow > 0 ? ("+" + overflow) : ""
  }

  function isSelected(value) {
    return selected.length > 0 && selected === String(value || "")
  }

  function selectValue(value) {
    selected = String(value || "")
    Qt.callLater(ensureSelectedVisible)
  }

  function ensureDefaultSelection() {
    // Guard: during early binding/setup selectableValues can briefly be undefined
    // and used to abort the whole Share dialog (TypeError on .length).
    var values = selectableValues
    if (!values || values.length === undefined) {
      selected = ""
      return
    }
    if (values.length === 0) {
      selected = ""
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
    // Best-effort: keep content moving when selection walks off-screen.
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

  // Defer so derived lists (selectableValues) are fully bound before we touch .length.
  onQueryChanged: Qt.callLater(ensureDefaultSelection)
  onSelectedGeometriesChanged: Qt.callLater(ensureDefaultSelection)
  onAllScreensChanged: Qt.callLater(ensureDefaultSelection)
  onAllWindowsChanged: Qt.callLater(ensureDefaultSelection)
  Component.onCompleted: {
    try { Hyprland.refreshToplevels() } catch (e) {}
    Qt.callLater(ensureDefaultSelection)
  }

  Keys.onPressed: function (e) {
    if (handleNavKey(e)) {
      e.accepted = true
      return
    }
    // Tab still toggles Cancel/Share via PortalDialog.
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

      Button {
        text: "Share region"
        bordered: true
        Layout.alignment: Qt.AlignVCenter
        onClicked: {
          root.selectValue("REGION_PICK")
          root.confirmSelection()
        }
      }

      Item {
        Layout.fillWidth: true
        visible: !chipsFlow.visible
      }

      Flow {
        id: chipsFlow
        Layout.fillWidth: true
        spacing: Style.space(6)
        visible: root.realScreens.length > 1

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
        Layout.preferredWidth: Style.space(220)
        Layout.alignment: Qt.AlignRight
        placeholderText: "Search…"
        text: root.query
        onTextChanged: root.query = text
        Keys.onPressed: function (e) {
          // Arrows navigate cards even while search is focused.
          if (root.handleNavKey(e))
            e.accepted = true
        }
      }
    }

    // Flickable + edge scrollbar slot (bar is attached for sync, parented to the
    // right gutter so it never overlays card previews).
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

          GridLayout {
            Layout.fillWidth: true
            columns: root.gridColumns
            columnSpacing: root.gridGap
            rowSpacing: root.gridGap
            visible: root.filteredScreens.length > 0

            Repeater {
              model: root.filteredScreens

              // Same card chrome / cell size as Windows (no full-row span).
              BorderSurface {
                id: displayCard
                required property var modelData
                readonly property var item: modelData
                readonly property bool sel: root.isSelected(item.value)
                readonly property bool hot: displayHover.hovered || sel
                readonly property var tasks: (item && item.tasks) ? item.tasks : []

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
                clip: true

                ColumnLayout {
                  anchors.fill: parent
                  anchors.margins: Style.space(8)
                  spacing: Style.space(8)

                  // Header matches window cards: icon + title (+ task icons).
                  RowLayout {
                    Layout.fillWidth: true
                    spacing: Style.space(8)

                    Image {
                      Layout.preferredWidth: Style.space(18)
                      Layout.preferredHeight: Style.space(18)
                      source: root.iconSource(displayCard.item.icon || "video-display")
                      fillMode: Image.PreserveAspectFit
                      asynchronous: true
                    }

                    Text {
                      Layout.fillWidth: true
                      text: root.field(displayCard.item, "name", "") || root.field(displayCard.item, "label", "Display")
                      color: displayCard.hot ? Color.menu.selectedText : Color.popups.text
                      font.family: Style.font.family
                      font.pixelSize: Style.font.body
                      font.bold: true
                      elide: Text.ElideRight
                    }

                    Repeater {
                      model: displayCard.tasks
                      delegate: Image {
                        required property var modelData
                        width: Style.space(16)
                        height: Style.space(16)
                        source: root.iconSource(modelData.icon)
                        fillMode: Image.PreserveAspectFit
                        asynchronous: true
                      }
                    }

                    Text {
                      visible: root.taskOverflowText(displayCard.item).length > 0
                      text: root.taskOverflowText(displayCard.item)
                      color: Color.muted
                      font.family: Style.font.family
                      font.pixelSize: Style.font.caption || Style.font.body
                    }
                  }

                  Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: Util.alpha(Color.popups.text, 0.06)
                    radius: Style.cornerRadius
                    clip: true

                    readonly property var captureSrc: displayCard.item.synthetic
                      ? null
                      : root.screenForName(displayCard.item.name)

                    ScreencopyView {
                      id: displayLive
                      anchors.fill: parent
                      anchors.margins: Style.space(4)
                      captureSource: parent.captureSrc
                      // Live for selected/hovered; still frame otherwise (cheaper).
                      live: displayCard.hot || displayCard.sel
                      paintCursor: false
                      visible: parent.captureSrc && hasContent
                      onCaptureSourceChanged: {
                        if (captureSource && !live)
                          Qt.callLater(captureFrame)
                      }
                      onLiveChanged: {
                        if (captureSource && !live)
                          Qt.callLater(captureFrame)
                      }
                      Component.onCompleted: {
                        if (captureSource && !live)
                          Qt.callLater(captureFrame)
                      }
                    }

                    Image {
                      anchors.centerIn: parent
                      width: Style.space(40)
                      height: Style.space(40)
                      visible: !displayLive.visible
                      source: root.iconSource(displayCard.item.icon || "video-display")
                      fillMode: Image.PreserveAspectFit
                      asynchronous: true
                    }
                  }
                }

                HoverHandler {
                  id: displayHover
                  cursorShape: Qt.PointingHandCursor
                }
                TapHandler {
                  onTapped: root.selectValue(String(displayCard.item.value || ""))
                  onDoubleTapped: {
                    root.selectValue(String(displayCard.item.value || ""))
                    root.confirmSelection()
                  }
                }
              }
            }
          }

          RowLayout {
            Layout.fillWidth: true
            spacing: Style.space(10)
            visible: root.showWindowsSeparator

            Rectangle {
              Layout.fillWidth: true
              Layout.preferredHeight: 1
              color: Util.alpha(Color.popups.text, 0.22)
            }
            Text {
              text: "Windows"
              color: Color.popups.text
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              font.bold: true
            }
            Rectangle {
              Layout.fillWidth: true
              Layout.preferredHeight: 1
              color: Util.alpha(Color.popups.text, 0.22)
            }
          }

          GridLayout {
            Layout.fillWidth: true
            columns: root.gridColumns
            columnSpacing: root.gridGap
            rowSpacing: root.gridGap
            visible: root.filteredWindows.length > 0

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
                      // Window toplevel-export is cheap enough to keep live.
                      live: !!windowCard.captureSrc
                      paintCursor: false
                      visible: !!windowCard.captureSrc && hasContent
                    }

                    Image {
                      anchors.centerIn: parent
                      width: Style.space(48)
                      height: Style.space(48)
                      visible: !winLive.visible
                      source: root.iconSource(windowCard.item.icon || windowCard.item.className)
                      fillMode: Image.PreserveAspectFit
                      asynchronous: true
                      opacity: 0.55
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
            visible: root.filteredScreens.length === 0 && root.filteredWindows.length === 0
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            text: root.query.length || root.selectedGeometries.length
              ? "No matching displays or windows"
              : "No displays or windows found"
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
        }
      }

      // Far-right gutter — never overlays the card column.
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
          // Drive from Flickable unless the user is dragging the bar.
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

  // Restore checkbox on the left of Cancel / Share.
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
