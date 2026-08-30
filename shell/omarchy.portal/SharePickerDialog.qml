import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

PortalDialog {
  id: root

  property var extra: ({})
  property var screens: []
  property var windows: extra.windows || []
  property string selected: ""
  property bool allowToken: extra.allowToken === true

  title: "Share screen"
  subtitle: "Choose a display, window, or region"
  acceptText: "Share"
  acceptable: selected.length > 0
  cardWidth: Style.space(520)
  cardHeight: Style.space(520)
  focus: true

  signal picked(string line)

  readonly property var items: {
    var out = []
    for (var i = 0; i < screens.length; i++)
      out.push(screens[i])
    for (var j = 0; j < windows.length; j++) {
      var w = windows[j]
      out.push({
        label: "Window · " + String(w.class || "") + (w.title ? ": " + w.title : ""),
        value: "window:" + String(w.id || "")
      })
    }
    out.push({ label: "Select region…", value: "REGION_PICK" })
    return out
  }

  onAccepted: if (selected) emitSelection(selected)

  function emitSelection(value) {
    if (value === "REGION_PICK") {
      picked("REGION_PICK")
      return
    }
    picked((allowToken ? "[SELECTION]r/" : "[SELECTION]/") + value)
  }

  Keys.onPressed: function(e) { if (handleKey(e)) e.accepted = true }

  Component.onCompleted: {
    monProc.running = true
  }

  Process {
    id: monProc
    command: ["hyprctl", "-j", "monitors"]
    stdout: StdioCollector {
      onStreamFinished: {
        try {
          var arr = JSON.parse(text) || []
          var out = []
          for (var i = 0; i < arr.length; i++)
            out.push({ label: "Display · " + arr[i].name, value: "screen:" + arr[i].name })
          root.screens = out
          if (!root.selected && out.length) root.selected = out[0].value
        } catch (e) {}
      }
    }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(8)

    ListView {
      id: list
      Layout.fillWidth: true
      Layout.fillHeight: true
      clip: true
      model: root.items
      delegate: Item {
        required property var modelData
        width: list.width
        height: Style.space(36)
        readonly property bool sel: root.selected === modelData.value
        Rectangle {
          anchors.fill: parent
          color: sel ? Style.selectedFillFor(Color.foreground, Color.accent) : "transparent"
          radius: Style.cornerRadius
          Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Style.space(10)
            text: modelData.label
            color: sel ? Color.accent : Color.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.body
            elide: Text.ElideRight
            width: parent.width - Style.space(20)
          }
        }
        MouseArea {
          anchors.fill: parent
          onClicked: root.selected = modelData.value
          onDoubleClicked: {
            root.selected = modelData.value
            root.emitSelection(root.selected)
          }
        }
      }
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
