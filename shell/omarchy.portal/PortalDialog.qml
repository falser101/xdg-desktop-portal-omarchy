import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Ui

// Shared chrome for every portal dialog. Tokens come from Style/Color so
// theme-set updates the look without a portal restart.
BorderSurface {
  id: root

  property string title: ""
  property string subtitle: ""
  property string cancelText: "Cancel"
  property string acceptText: "OK"
  property bool acceptable: true
  property bool showButtons: true
  property int selectedIndex: 1
  default property alias body: bodySlot.data
  property real cardWidth: Math.min(Style.space(560), (parent ? parent.width : 800) - Style.gapsOut * 4)
  property real cardHeight: Math.min(Style.space(520), (parent ? parent.height : 600) - Style.gapsOut * 4)

  signal accepted()
  signal rejected()

  width: parent ? parent.width : cardWidth
  height: parent ? parent.height : cardHeight
  implicitWidth: cardWidth
  implicitHeight: cardHeight
  color: Color.background
  radius: Style.cornerRadius
  borderSpec: Border.flat(Color.accent, Style.normalBorderWidth)
  padding: Style.space(18)

  function handleKey(event) {
    if (event.key === Qt.Key_Escape) {
      root.rejected()
      return true
    }
    if (event.key === Qt.Key_Left || event.key === Qt.Key_Right || event.key === Qt.Key_Tab || event.key === Qt.Key_Backtab) {
      root.selectedIndex = root.selectedIndex === 0 ? 1 : 0
      return true
    }
    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
      if (root.selectedIndex === 0) root.rejected()
      else if (root.acceptable) root.accepted()
      return true
    }
    return false
  }

  ColumnLayout {
    anchors.fill: parent
    anchors.topMargin: root.contentTopInset
    anchors.rightMargin: root.contentRightInset
    anchors.bottomMargin: root.contentBottomInset
    anchors.leftMargin: root.contentLeftInset
    spacing: Style.space(12)

    Text {
      Layout.fillWidth: true
      text: root.title
      color: Color.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.title
      wrapMode: Text.WordWrap
    }

    Text {
      Layout.fillWidth: true
      visible: root.subtitle.length > 0
      text: root.subtitle
      color: Color.muted
      font.family: Style.font.family
      font.pixelSize: Style.font.body
      wrapMode: Text.WordWrap
    }

    Item {
      id: bodySlot
      Layout.fillWidth: true
      Layout.fillHeight: true
    }

    Row {
      visible: root.showButtons
      Layout.alignment: Qt.AlignRight
      spacing: Style.space(10)

      Repeater {
        model: [root.cancelText, root.acceptText]
        Button {
          required property int index
          required property string modelData
          text: modelData
          selected: root.selectedIndex === index
          bordered: true
          enabled: index === 0 || root.acceptable
          onClicked: {
            root.selectedIndex = index
            if (index === 0) root.rejected()
            else root.accepted()
          }
        }
      }
    }
  }
}
