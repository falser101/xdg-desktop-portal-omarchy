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
          }
        }
        MouseArea {
          anchors.fill: parent
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
