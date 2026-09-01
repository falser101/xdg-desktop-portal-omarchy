import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

// Account dialog: title + privacy subtitle, large centered
// avatar, real name, username, Share / Cancel.
PortalDialog {
  id: root

  property var request: ({})
  property var extra: ({})

  signal decided(bool shared)

  title: String(request.title || "Share user info with this application?")
  subtitle: String(request.subtitle || "")
  acceptText: "Share"
  // Tall card for the avatar stack.
  cardWidth: Style.space(420)
  cardHeight: Style.space(480)
  focus: true

  readonly property string username: String(request.username || extra.user || "")
  readonly property string realName: String(request.real_name || extra.realName || "")
  readonly property string avatarPath: String(request.image || extra.image || "")

  function avatarSource() {
    var value = String(avatarPath || "")
    if (value.indexOf("file://") === 0 || value.indexOf("image://") === 0)
      return value
    if (value.charAt(0) === "/")
      return "file://" + value
    if (value.length > 0) {
      var themed = Quickshell.iconPath(value, true)
      if (themed && themed.length)
        return themed
    }
    var userIcon = Quickshell.iconPath("user", true)
    if (userIcon && userIcon.length)
      return userIcon
    var identity = Quickshell.iconPath("user-identity", true)
    if (identity && identity.length)
      return identity
    return Quickshell.iconPath("avatar-default", true)
  }

  onAccepted: root.decided(true)
  onRejected: root.decided(false)

  Keys.onPressed: function(e) {
    if (handleKey(e))
      e.accepted = true
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(10)

    Item {
      Layout.fillHeight: true
    }

    Item {
      id: avatarFrame
      readonly property int size: Style.space(128)
      Layout.preferredWidth: size
      Layout.preferredHeight: size
      Layout.alignment: Qt.AlignHCenter

      Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: Util.alpha(Color.popups.text, 0.08)
        border.width: Style.normalBorderWidth
        border.color: Util.alpha(Color.popups.border, 0.7)
        clip: true

        Image {
          id: avatar
          anchors.fill: parent
          source: root.avatarSource()
          fillMode: Image.PreserveAspectCrop
          asynchronous: true
          sourceSize.width: width * Screen.devicePixelRatio
          sourceSize.height: height * Screen.devicePixelRatio
          visible: status === Image.Ready
          onStatusChanged: {
            if (status === Image.Error) {
              var fb = Quickshell.iconPath("user", true)
              if (fb && fb.length && source !== fb)
                source = fb
            }
          }
        }

        Text {
          anchors.centerIn: parent
          visible: avatar.status !== Image.Ready
          text: root.realName.length ? root.realName.charAt(0).toUpperCase()
                : (root.username.length ? root.username.charAt(0).toUpperCase() : "?")
          color: Color.popups.text
          font.family: Style.font.family
          font.pixelSize: Style.font.title
          font.bold: true
        }
      }
    }

    Text {
      Layout.fillWidth: true
      Layout.alignment: Qt.AlignHCenter
      visible: root.realName.length > 0
      text: root.realName
      color: Color.popups.text
      font.family: Style.font.family
      font.pixelSize: Style.font.title
      font.bold: true
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
    }

    Text {
      Layout.fillWidth: true
      Layout.topMargin: -Style.space(4)
      Layout.alignment: Qt.AlignHCenter
      visible: root.username.length > 0
      text: root.username
      color: Util.alpha(Color.popups.text, 0.55)
      font.family: Style.font.family
      font.pixelSize: Style.font.body
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
    }

    Item {
      Layout.fillHeight: true
    }
  }
}
