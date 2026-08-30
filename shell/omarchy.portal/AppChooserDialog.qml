import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

PortalDialog {
  id: root

  property var request: ({})
  property var extra: ({})

  signal chosen(string choice)

  title: String(request.title || "Open with")
  subtitle: String(request.uri || request.filename || request.content_type || "")
  acceptText: "Open"
  acceptable: selectedId.length > 0
  cardWidth: Style.space(480)
  cardHeight: Style.space(520)
  focus: true

  property string query: ""
  property string selectedId: {
    var last = String(request.last_choice || "")
    var apps = extra.apps || []
    for (var i = 0; i < apps.length; i++) {
      if (String(apps[i].id) === last) return last
    }
    return apps.length ? String(apps[0].id) : ""
  }
  readonly property string lastChoice: String(request.last_choice || "")

  function filteredApps() {
    var apps = extra.apps || []
    var q = query.toLowerCase()
    if (!q) return apps
    var out = []
    for (var i = 0; i < apps.length; i++) {
      var a = apps[i]
      var hay = (String(a.name || "") + " " + String(a.id || "")).toLowerCase()
      if (hay.indexOf(q) !== -1) out.push(a)
    }
    return out
  }

  function iconSource(icon) {
    var value = String(icon || "")
    if (value.length === 0) return Quickshell.iconPath("application-x-executable", true)
    if (value.indexOf("file://") === 0 || value.indexOf("image://") === 0) return value
    if (value.charAt(0) === "/") return "file://" + value
    var themed = Quickshell.iconPath(value, true)
    if (themed && themed.length) return themed
    return Quickshell.iconPath("application-x-executable", true)
  }

  onAccepted: if (selectedId) root.chosen(selectedId)

  Keys.onPressed: function(e) {
    if (handleKey(e)) { e.accepted = true; return }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(8)

    TextField {
      Layout.fillWidth: true
      placeholderText: "Search applications"
      onTextChanged: root.query = text
    }

    ListView {
      id: list
      Layout.fillWidth: true
      Layout.fillHeight: true
      clip: true
      model: root.filteredApps()
      currentIndex: 0
      delegate: Item {
        required property var modelData
        required property int index
        width: list.width
        height: Style.space(40)
        readonly property bool sel: String(modelData.id) === root.selectedId
        readonly property bool isDefault: String(modelData.id) === root.lastChoice && root.lastChoice.length > 0

        Rectangle {
          anchors.fill: parent
          color: sel ? Style.selectedFillFor(Color.foreground, Color.accent) : "transparent"
          radius: Style.cornerRadius

          Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: Style.space(10)
            anchors.rightMargin: Style.space(10)
            spacing: Style.space(10)

            Image {
              width: Style.space(22)
              height: Style.space(22)
              fillMode: Image.PreserveAspectFit
              sourceSize.width: width * Screen.devicePixelRatio
              sourceSize.height: height * Screen.devicePixelRatio
              source: root.iconSource(modelData.icon)
              asynchronous: true
            }

            Text {
              width: parent.width - Style.space(80)
              text: String(modelData.name || modelData.id)
              color: sel ? Color.accent : Color.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              elide: Text.ElideRight
              anchors.verticalCenter: parent.verticalCenter
            }

            Text {
              visible: isDefault
              text: "Default"
              color: Color.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.caption || Style.font.body
              anchors.verticalCenter: parent.verticalCenter
            }
          }
        }

        MouseArea {
          anchors.fill: parent
          onClicked: root.selectedId = String(modelData.id)
          onDoubleClicked: {
            root.selectedId = String(modelData.id)
            root.chosen(root.selectedId)
          }
        }
      }
    }
  }
}
