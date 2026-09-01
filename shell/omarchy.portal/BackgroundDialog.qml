import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Ui

// KDE Background uses a notification with Allow / Deny (+ Allow once when
 // dismissed). Omarchy toasts cannot show action buttons, so we surface the
 // same three outcomes in a dialog.
PortalDialog {
  id: root

  property var request: ({})

  // 0 Forbid, 1 Allow, 2 Allow once (xdg-desktop-portal Background)
  signal decided(int result)

  title: String(request.title || "Background Activity")
  subtitle: String(request.subtitle || "")
  showButtons: false
  cardWidth: Style.space(460)
  cardHeight: Style.space(260)
  focus: true

  function pick(result) {
    root.decided(result)
  }

  function handleKey(event) {
    if (event.key === Qt.Key_Escape) {
      // Match KDE: dismiss without choosing → Allow once
      root.pick(2)
      return true
    }
    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
      root.pick(1)
      return true
    }
    return false
  }

  Keys.onPressed: function(e) {
    if (handleKey(e))
      e.accepted = true
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(14)

    Item { Layout.fillHeight: true }

    Text {
      Layout.fillWidth: true
      visible: String(request.body || "").length > 0
      text: String(request.body || "")
      color: Util.alpha(Color.popups.text, 0.75)
      font.family: Style.font.family
      font.pixelSize: Style.font.body
      wrapMode: Text.WordWrap
    }

    Item { Layout.fillHeight: true }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.space(10)

      Button {
        text: "Deny"
        bordered: true
        onClicked: root.pick(0)
      }

      Item { Layout.fillWidth: true }

      Button {
        text: "Allow once"
        bordered: true
        onClicked: root.pick(2)
      }

      Button {
        text: "Allow"
        bordered: true
        selected: true
        onClicked: root.pick(1)
      }
    }
  }
}
