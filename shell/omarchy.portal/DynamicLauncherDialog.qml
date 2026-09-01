import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

// DynamicLauncher layout: title + subtitle, large centered icon,
// name (and optional webapp URL). Edit Info toggles name editing.
PortalDialog {
  id: root

  property var request: ({})
  property var extra: ({})
  property bool editing: false
  property string editedName: ""

  signal decided(bool accepted, string name)

  readonly property string initialName: String(request.name || extra.name || "")
  readonly property string iconPath: String(request.icon_path || extra.iconPath || "")
  readonly property string targetUrl: String(request.target || extra.target || "")
  readonly property bool editableName: request.editable_name !== false
    && extra.editableName !== false

  title: String(request.main_text || "Create Application Launcher?")
  subtitle: String(request.subtitle || "")
  acceptText: "Create"
  cardWidth: Style.space(420)
  cardHeight: editing ? Style.space(460) : Style.space(440)
  focus: true
  acceptable: String(editedName || initialName).trim().length > 0

  Component.onCompleted: editedName = initialName

  function iconSource() {
    var value = String(iconPath || "")
    if (value.indexOf("file://") === 0 || value.indexOf("image://") === 0)
      return value
    if (value.charAt(0) === "/")
      return "file://" + value
    if (value.length > 0) {
      var themed = Quickshell.iconPath(value, true)
      if (themed && themed.length)
        return themed
    }
    var app = Quickshell.iconPath("application-x-executable", true)
    if (app && app.length)
      return app
    return Quickshell.iconPath("applications-other", true)
  }

  function displayName() {
    var n = String(editedName || initialName).trim()
    return n.length ? n : initialName
  }

  function acceptCreate() {
    root.decided(true, root.displayName())
  }

  onAccepted: root.acceptCreate()
  onRejected: root.decided(false, root.displayName())

  function handleKey(event) {
    if (event.key === Qt.Key_Escape) {
      root.rejected()
      return true
    }
    if (root.editing && nameField.activeFocus)
      return false
    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
      if (root.acceptable)
        root.acceptCreate()
      return true
    }
    return false
  }

  Keys.onPressed: function(e) {
    if (handleKey(e))
      e.accepted = true
  }

  footerLeft: Button {
    visible: root.editableName
    text: root.editing ? "Done" : "Edit Info…"
    bordered: true
    selected: root.editing
    onClicked: {
      root.editing = !root.editing
      if (root.editing)
        Qt.callLater(function() { nameField.forceActiveFocus() })
    }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(10)

    Item { Layout.fillHeight: true }

    Item {
      readonly property int size: Style.space(96)
      Layout.preferredWidth: size
      Layout.preferredHeight: size
      Layout.alignment: Qt.AlignHCenter

      Rectangle {
        anchors.fill: parent
        radius: Style.cornerRadius
        color: Util.alpha(Color.popups.text, 0.06)
        border.width: Style.normalBorderWidth
        border.color: Util.alpha(Color.popups.border, 0.7)
        clip: true

        Image {
          id: iconImage
          anchors.fill: parent
          anchors.margins: Style.space(8)
          source: root.iconSource()
          fillMode: Image.PreserveAspectFit
          asynchronous: true
          sourceSize.width: width * Screen.devicePixelRatio
          sourceSize.height: height * Screen.devicePixelRatio
          visible: status === Image.Ready
          onStatusChanged: {
            if (status === Image.Error) {
              var fb = Quickshell.iconPath("application-x-executable", true)
              if (fb && fb.length && source !== fb)
                source = fb
            }
          }
        }

        Text {
          anchors.centerIn: parent
          visible: iconImage.status !== Image.Ready
          text: root.displayName().length ? root.displayName().charAt(0).toUpperCase() : "?"
          color: Color.popups.text
          font.family: Style.font.family
          font.pixelSize: Style.font.title
          font.bold: true
        }
      }
    }

    // Display mode: name under icon
    Text {
      Layout.fillWidth: true
      Layout.alignment: Qt.AlignHCenter
      visible: !root.editing
      text: root.displayName()
      color: Color.popups.text
      font.family: Style.font.family
      font.pixelSize: Style.font.title
      font.bold: true
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
    }

    // Edit mode: Name label + field
    ColumnLayout {
      Layout.fillWidth: true
      spacing: Style.space(6)
      visible: root.editing

      Text {
        Layout.fillWidth: true
        text: "Name"
        color: Util.alpha(Color.popups.text, 0.7)
        font.family: Style.font.family
        font.pixelSize: Style.font.body
      }

      TextField {
        id: nameField
        Layout.fillWidth: true
        text: root.editedName
        onTextChanged: root.editedName = text
        onAccepted: {
          if (root.acceptable)
            root.acceptCreate()
        }
      }
    }

    Text {
      Layout.fillWidth: true
      Layout.alignment: Qt.AlignHCenter
      visible: root.targetUrl.length > 0 && !root.editing
      text: root.targetUrl
      color: Color.menu.selectedBorder
      font.family: Style.font.family
      font.pixelSize: Style.font.body
      horizontalAlignment: Text.AlignHCenter
      elide: Text.ElideMiddle
      wrapMode: Text.NoWrap

      MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: Qt.openUrlExternally(root.targetUrl)
      }
    }

    Item { Layout.fillHeight: true }
  }
}
