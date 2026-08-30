import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

Item {
  id: root

  property var shell: null
  property var manifest: null
  property bool opened: false
  property string kind: ""
  property var request: ({})
  property string replyFile: ""
  property string doneFile: ""
  property var extra: ({})
  property bool finishing: false

  function open(payload) {
    var args = {}
    if (payload) {
      try { args = JSON.parse(payload) || {} } catch (e) { args = {} }
    }
    kind = String(args.kind || "")
    request = args.request || {}
    extra = args.extra || {}
    replyFile = String(args.replyFile || "")
    doneFile = String(args.doneFile || "")
    opened = true
  }

  function close() {
    if (opened && doneFile)
      finish({ "kind": "Cancel" })
    else
      opened = false
  }

  function finish(replyObj) {
    if (!opened || finishing) return
    finishing = true
    opened = false
    var json = JSON.stringify(replyObj || { "kind": "Cancel" })
    writer.replyJson = json
    writer.replyPath = replyFile
    writer.donePath = doneFile
    writer.running = false
    writer.running = true
    finishing = false
  }

  function acceptAccess() {
    finish({ "kind": "Access", "granted": true, "choices": [] })
  }

  function acceptAccount() {
    finish({
      "kind": "Account",
      "id": String(request.id || extra.user || ""),
      "name": String(request.name || extra.realName || ""),
      "image": extra.image || null
    })
  }

  function acceptWallpaper() {
    finish({ "kind": "Wallpaper", "granted": true })
  }

  function acceptApp(choice) {
    finish({ "kind": "App", "choice": String(choice || "") })
  }

  function acceptFiles(paths, choices, currentFilter) {
    finish({
      "kind": "FileChooser",
      "paths": paths || [],
      "choices": choices || [],
      "current_filter": currentFilter || null
    })
  }

  function acceptShare(selectionLine) {
    finish({ "kind": "Share", "selection": String(selectionLine || "") })
  }

  function acceptConfirm() {
    finish({ "kind": "Confirm", "accepted": true })
  }

  function acceptScreenshot(target) {
    finish({ "kind": "Screenshot", "target": Number(target) })
  }

  Process {
    id: writer
    property string replyJson: ""
    property string replyPath: ""
    property string donePath: ""
    command: ["bash", "-c",
      "umask 077; printf '%s' \"$1\" > \"$2\"; : > \"$3\"",
      "omarchy-portal-write", replyJson, replyPath, donePath]
  }

  // KDE desktop portals are a normal Qt.Dialog window, not a layer-shell
  // overlay. Match that: a centered floating XDG toplevel sized to the card.
  FloatingWindow {
    id: panel
    visible: root.opened
    title: "Omarchy Portal"
    color: Color.background
    implicitWidth: dialogLoader.item && dialogLoader.item.cardWidth ? dialogLoader.item.cardWidth : 480
    implicitHeight: dialogLoader.item && dialogLoader.item.cardHeight ? dialogLoader.item.cardHeight : 280
    minimumSize: Qt.size(320, 180)

    onVisibleChanged: {
      if (!visible && root.opened && !root.finishing)
        root.finish({ "kind": "Cancel" })
    }

    FocusScope {
      anchors.fill: parent
      focus: panel.visible
      Keys.onPressed: function(event) {
        var item = dialogLoader.item
        if (item && typeof item.handleKey === "function" && item.handleKey(event))
          event.accepted = true
      }

      Loader {
        id: dialogLoader
        anchors.fill: parent
        active: root.opened
        sourceComponent: {
          switch (root.kind) {
          case "Access": return accessComp
          case "Account": return accountComp
          case "Wallpaper": return wallpaperComp
          case "AppChooser": return appComp
          case "FileChooser": return fileComp
          case "Share": return shareComp
          case "Confirm": return confirmComp
          case "Screenshot": return screenshotComp
          default: return accessComp
          }
        }
      }
    }
  }

  Component {
    id: accessComp
    PortalDialog {
      title: String(root.request.title || "Allow access?")
      subtitle: [root.request.subtitle, root.request.body].filter(function(s) { return s }).join("\n")
      cancelText: String(root.request.deny_label || "Deny")
      acceptText: String(root.request.grant_label || "Allow")
      cardWidth: Style.space(420)
      cardHeight: Style.space(220)
      onAccepted: root.acceptAccess()
      onRejected: root.finish({ "kind": "Cancel" })
      Keys.onPressed: function(e) { if (handleKey(e)) e.accepted = true }
      focus: true
    }
  }

  Component {
    id: accountComp
    PortalDialog {
      title: String(root.request.title || "Share account information")
      subtitle: String(root.request.reason || "")
      acceptText: "Allow"
      cardWidth: Style.space(420)
      cardHeight: Style.space(240)
      onAccepted: root.acceptAccount()
      onRejected: root.finish({ "kind": "Cancel" })
      Keys.onPressed: function(e) { if (handleKey(e)) e.accepted = true }
      focus: true
      Text {
        width: parent.width
        text: "User: " + String(root.extra.user || "") + "\nName: " + String(root.extra.realName || "")
        color: Color.foreground
        font.family: Style.font.family
        font.pixelSize: Style.font.body
      }
    }
  }

  Component {
    id: wallpaperComp
    PortalDialog {
      title: "Set Omarchy wallpaper?"
      subtitle: String(root.request.uri || root.extra.uri || "")
      acceptText: "Set"
      cardWidth: Style.space(480)
      cardHeight: Style.space(200)
      onAccepted: root.acceptWallpaper()
      onRejected: root.finish({ "kind": "Cancel" })
      Keys.onPressed: function(e) { if (handleKey(e)) e.accepted = true }
      focus: true
    }
  }

  Component {
    id: appComp
    AppChooserDialog {
      request: root.request
      extra: root.extra
      onChosen: function(choice) { root.acceptApp(choice) }
      onRejected: root.finish({ "kind": "Cancel" })
    }
  }

  Component {
    id: fileComp
    FileChooserDialog {
      request: root.request
      extra: root.extra
      onPicked: function(paths, choices, currentFilter) {
        root.acceptFiles(paths, choices || [], currentFilter || null)
      }
      onRejected: root.finish({ "kind": "Cancel" })
    }
  }

  Component {
    id: confirmComp
    PortalDialog {
      title: String(root.request.title || "Continue?")
      subtitle: String(root.request.subtitle || "")
      acceptText: String(root.request.accept || "OK")
      cardWidth: Style.space(420)
      cardHeight: Style.space(200)
      onAccepted: root.acceptConfirm()
      onRejected: root.finish({ "kind": "Cancel" })
      Keys.onPressed: function(e) { if (handleKey(e)) e.accepted = true }
      focus: true
    }
  }

  Component {
    id: screenshotComp
    ScreenshotDialog {
      onPicked: function(target) { root.acceptScreenshot(target) }
      onRejected: root.finish({ "kind": "Cancel" })
    }
  }

  Component {
    id: shareComp
    SharePickerDialog {
      extra: root.extra
      onPicked: function(line) { root.acceptShare(line) }
      onRejected: root.finish({ "kind": "Cancel" })
    }
  }
}
