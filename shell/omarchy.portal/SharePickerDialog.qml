import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

PortalDialog {
  id: root

  property var extra: ({})
  property string page: "displays"
  property string query: ""
  property string selected: ""
  property bool allowToken: extra.allowToken === true
  property int previewRev: 0

  readonly property var screens: extra.screens || []
  readonly property var windows: extra.windows || []

  title: "Share screen"
  subtitle: "Choose a display, window, or region"
  acceptText: "Share"
  acceptable: selected.length > 0
  cardWidth: Style.space(920)
  cardHeight: Style.space(640)
  focus: true

  signal picked(string line, bool allowToken)

  readonly property var visibleItems: {
    var _ = previewRev
    if (page === "displays")
      return screens
    var q = query.toLowerCase()
    if (!q)
      return windows
    var out = []
    for (var i = 0; i < windows.length; i++) {
      var w = windows[i]
      var hay = (String(w.label || "") + " " + String(w.className || "") + " " + String(w.title || "")).toLowerCase()
      if (hay.indexOf(q) !== -1)
        out.push(w)
    }
    return out
  }

  onAccepted: if (selected)
    emitSelection(selected)

  function emitSelection(value) {
    if (value === "REGION_PICK") {
      picked("REGION_PICK", allowToken)
      return
    }
    picked((allowToken ? "[SELECTION]r/" : "[SELECTION]/") + value, allowToken)
  }

  function previewUrl(path) {
    var p = String(path || "")
    if (!p.length)
      return ""
    if (p.indexOf("file:") === 0)
      return p + "?r=" + previewRev
    return "file://" + p + "?r=" + previewRev
  }

  function setPage(next) {
    page = next
    selected = ""
    query = ""
  }

  Keys.onPressed: function (e) {
    if (handleKey(e))
      e.accepted = true
  }

  Component.onCompleted: {
    if (!selected && screens.length)
      selected = String(screens[0].value || "")
  }

  // Refresh Image sources once grim finishes writing files (share-picker
  // captures in parallel before summon; this covers late writes).
  Timer {
    interval: 350
    running: true
    repeat: true
    onTriggered: {
      root.previewRev += 1
      if (root.previewRev > 8)
        running = false
    }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(10)

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.space(8)

      Button {
        text: "Displays"
        bordered: true
        selected: root.page === "displays"
        onClicked: root.setPage("displays")
      }
      Button {
        text: "Windows"
        bordered: true
        selected: root.page === "windows"
        onClicked: root.setPage("windows")
      }

      Item {
        Layout.fillWidth: true
      }

      TextField {
        visible: root.page === "windows"
        Layout.preferredWidth: Style.space(240)
        placeholderText: "Search windows"
        text: root.query
        onTextChanged: root.query = text
      }
    }

    GridView {
      id: grid
      Layout.fillWidth: true
      Layout.fillHeight: true
      clip: true
      cellWidth: Style.space(280)
      cellHeight: Style.space(200)
      model: root.visibleItems

      delegate: Item {
        width: grid.cellWidth
        height: grid.cellHeight
        required property var modelData

        readonly property string value: String(modelData.value || "")
        readonly property bool sel: root.selected === value
        readonly property bool hot: sel || cardHover.containsMouse

        BorderSurface {
          anchors.fill: parent
          anchors.margins: Style.space(6)
          color: hot ? Color.menu.selectedBackground : Color.popups.background
          borderSpec: hot
            ? Border.surfaceSpec("menu", "selected-border", Color.menu.selectedBorder, Style.normalBorderWidth)
            : Border.flat(Util.alpha(Color.popups.text, 0.18), 1)
          radius: Style.cornerRadius
          padding: Style.space(10)

          ColumnLayout {
            anchors.fill: parent
            anchors.margins: Style.space(10)
            spacing: Style.space(8)

            Rectangle {
              Layout.fillWidth: true
              Layout.fillHeight: true
              color: Util.alpha(Color.popups.text, 0.06)
              radius: Style.cornerRadius
              clip: true

              Image {
                id: thumb
                anchors.fill: parent
                anchors.margins: Style.space(4)
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: false
                source: root.previewUrl(modelData.preview)
                sourceSize.width: 480
                sourceSize.height: 320
                visible: status === Image.Ready
              }

              Text {
                anchors.centerIn: parent
                visible: thumb.status !== Image.Ready
                text: root.page === "displays" ? "Display" : "Window"
                color: Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.caption || Style.font.body
              }
            }

            Text {
              Layout.fillWidth: true
              text: String(modelData.label || modelData.value || "")
              color: hot ? Color.menu.selectedText : Color.popups.text
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              elide: Text.ElideRight
              maximumLineCount: 2
              wrapMode: Text.WrapAnywhere
            }
          }
        }

        MouseArea {
          id: cardHover
          anchors.fill: parent
          hoverEnabled: true
          cursorShape: Qt.PointingHandCursor
          onClicked: root.selected = value
          onDoubleClicked: {
            root.selected = value
            root.emitSelection(value)
          }
        }
      }
    }

    Text {
      visible: root.visibleItems.length === 0
      Layout.fillWidth: true
      horizontalAlignment: Text.AlignHCenter
      text: root.page === "windows" ? "No windows to share" : "No displays found"
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.space(8)

      Button {
        text: "Select region…"
        bordered: true
        onClicked: root.emitSelection("REGION_PICK")
      }

      Toggle {
        Layout.fillWidth: true
        label: "Allow restore token"
        description: "Let this app share again without asking next time"
        checked: root.allowToken
        onClicked: root.allowToken = !root.allowToken
      }
    }
  }
}
