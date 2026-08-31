import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

PortalDialog {
  id: root

  property var request: ({})

  signal decided(bool granted, var choices)

  title: String(request.title || "Allow access?")
  subtitle: {
    var parts = []
    if (request.subtitle) parts.push(String(request.subtitle))
    if (request.body) parts.push(String(request.body))
    return parts.join("\n")
  }
  cancelText: String(request.deny_label || "Deny")
  acceptText: String(request.grant_label || "Allow")
  cardWidth: Style.space(440)
  cardHeight: Style.space(choices.length > 0 ? 320 : 220)
  focus: true

  readonly property var choices: request.choices || []
  readonly property string iconName: String(request.icon || "dialog-question")
  property var choiceValues: initialChoiceValues()

  function initialChoiceValues() {
    var initial = ({})
    var list = choices
    for (var i = 0; i < list.length; i++) {
      var c = list[i]
      var id = String(c.id || i)
      var selected = String(c.selected || "")
      if (!selected) {
        if (choiceHasOptions(c) && c.options && c.options.length)
          selected = optionPair(c.options[0], 0).value
        else
          selected = "false"
      }
      initial[id] = selected
    }
    return initial
  }

  function choiceHasOptions(c) {
    return !!(c && c.options && c.options.length)
  }

  // Quickshell/QVariant often turns JSON pairs into objects with "0"/"1"
  // keys instead of real JS arrays, so Array.isArray fails and we used to
  // fall back to the loop index (dropdown showed "0" / "1").
  function optionPair(o, index) {
    if (o === undefined || o === null)
      return { value: String(index), label: String(index) }
    if (Array.isArray(o))
      return { value: String(o[0]), label: String(o[1] || o[0]) }
    if (typeof o === "object") {
      if (o.value !== undefined || o.label !== undefined || o.id !== undefined) {
        var v = o.value !== undefined ? o.value : (o.id !== undefined ? o.id : index)
        var l = o.label !== undefined ? o.label : v
        return { value: String(v), label: String(l) }
      }
      if (o[0] !== undefined || o["0"] !== undefined) {
        var pv = o[0] !== undefined ? o[0] : o["0"]
        var pl = o[1] !== undefined ? o[1] : o["1"]
        return { value: String(pv), label: String(pl !== undefined ? pl : pv) }
      }
    }
    return { value: String(o), label: String(o) }
  }

  function choiceOptions(c) {
    var out = []
    var opts = (c && c.options) || []
    for (var i = 0; i < opts.length; i++)
      out.push(optionPair(opts[i], i))
    return out
  }

  function selectedChoices() {
    var out = []
    for (var i = 0; i < choices.length; i++) {
      var id = String(choices[i].id || i)
      var val = choiceValues[id]
      if (val === undefined) val = choices[i].selected || "false"
      out.push([id, String(val)])
    }
    return out
  }

  function iconSource(icon) {
    var value = String(icon || "")
    if (value.indexOf("file://") === 0 || value.indexOf("image://") === 0) return value
    if (value.charAt(0) === "/") return "file://" + value
    var themed = Quickshell.iconPath(value, true)
    if (themed && themed.length) return themed
    return Quickshell.iconPath("dialog-question", true)
  }

  onAccepted: root.decided(true, selectedChoices())
  onRejected: root.decided(false, [])

  Keys.onPressed: function(e) {
    if (handleKey(e)) { e.accepted = true; return }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.space(12)

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.space(12)
      visible: root.iconName.length > 0

      Image {
        id: accessIcon
        Layout.preferredWidth: Style.space(36)
        Layout.preferredHeight: Style.space(36)
        fillMode: Image.PreserveAspectFit
        sourceSize.width: width * Screen.devicePixelRatio
        sourceSize.height: height * Screen.devicePixelRatio
        source: root.iconSource(root.iconName)
        asynchronous: true
        onStatusChanged: {
          if (status === Image.Error) {
            var fb = Quickshell.iconPath("dialog-question", true)
            if (source !== fb) source = fb
          }
        }
      }

      Item { Layout.fillWidth: true }
    }

    ColumnLayout {
      Layout.fillWidth: true
      Layout.fillHeight: true
      spacing: Style.space(10)
      visible: root.choices.length > 0

      Repeater {
        model: root.choices
        delegate: RowLayout {
          required property var modelData
          required property int index
          Layout.fillWidth: true
          spacing: Style.space(10)

          Text {
            Layout.fillWidth: true
            text: String(modelData.label || modelData.id)
            color: Color.popups.text
            font.family: Style.font.family
            font.pixelSize: Style.font.body
            wrapMode: Text.WordWrap
          }

          UpDropdown {
            visible: root.choiceHasOptions(modelData)
            Layout.preferredWidth: Style.space(160)
            showLabel: false
            value: String(root.choiceValues[String(modelData.id || index)] || modelData.selected || "")
            options: root.choiceOptions(modelData)
            onChanged: function(v) {
              var next = Object.assign({}, root.choiceValues)
              next[String(modelData.id || index)] = v
              root.choiceValues = next
            }
          }

          ToggleSwitch {
            visible: !root.choiceHasOptions(modelData)
            checked: String(root.choiceValues[String(modelData.id || index)] || modelData.selected) === "true"
            onToggled: {
              var next = Object.assign({}, root.choiceValues)
              next[String(modelData.id || index)] = checked ? "false" : "true"
              root.choiceValues = next
            }
          }
        }
      }
    }

    Item {
      Layout.fillWidth: true
      Layout.fillHeight: true
      visible: root.choices.length === 0
    }
  }
}
