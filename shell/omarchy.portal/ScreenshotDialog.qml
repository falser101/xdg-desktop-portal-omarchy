import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Ui

PortalDialog {
  id: root

  title: "Take a screenshot?"
  subtitle: "Choose what to capture"
  acceptText: "Capture"
  acceptable: selected !== 0
  cardWidth: Style.space(420)
  cardHeight: Style.space(280)
  focus: true

  property int selected: 1
  signal picked(int target)

  readonly property var targets: [
    { label: "Entire screen", value: 1 },
    { label: "Select area", value: 4 },
    { label: "Active window", value: 8 }
  ]

  onAccepted: if (selected) picked(selected)

  Keys.onPressed: function(e) {
    if (e.key === Qt.Key_Up || e.key === Qt.Key_Down) {
      var i = 0
      for (var n = 0; n < targets.length; n++)
        if (targets[n].value === selected) i = n
      i = (i + (e.key === Qt.Key_Down ? 1 : targets.length - 1)) % targets.length
      selected = targets[i].value
      e.accepted = true
      return
    }
    if (handleKey(e)) e.accepted = true
  }

  Column {
    anchors.fill: parent
    spacing: Style.space(4)

    Repeater {
      model: root.targets
      delegate: Item {
        required property var modelData
        width: parent.width
        height: Style.space(36)
        readonly property bool sel: root.selected === modelData.value
        readonly property bool hot: sel || shotHover.containsMouse
        BorderSurface {
          anchors.fill: parent
          color: hot ? Color.menu.selectedBackground : "transparent"
          borderSpec: hot ? Border.surfaceSpec("menu", "selected-border", Color.menu.selectedBorder, 0) : Border.none()
          radius: Style.cornerRadius
          Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Style.space(10)
            text: modelData.label
            color: hot ? Color.menu.selectedText : Color.popups.text
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }
        }
        MouseArea {
          id: shotHover
          anchors.fill: parent
          hoverEnabled: true
          cursorShape: Qt.PointingHandCursor
          onClicked: root.selected = modelData.value
          onDoubleClicked: {
            root.selected = modelData.value
            root.picked(root.selected)
          }
        }
      }
    }
  }
}
