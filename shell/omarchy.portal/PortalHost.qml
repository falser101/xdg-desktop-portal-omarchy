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

  function acceptAccess(choices) {
    finish({
      "kind": "Access",
      "granted": true,
      "choices": choices || []
    })
  }

  function acceptAccount() {
    var id = String(request.username || extra.user || "")
    var name = String(request.real_name || extra.realName || id)
    var image = request.image || extra.image || null
    finish({
      "kind": "Account",
      "id": id,
      "name": name,
      "image": image
    })
  }

  function acceptWallpaper() {
    finish({ "kind": "Wallpaper", "granted": true })
  }

  function acceptApp(choice, remember) {
    finish({
      "kind": "App",
      "choice": String(choice || ""),
      "remember": !!remember
    })
  }

  function acceptFiles(paths, choices, currentFilter) {
    finish({
      "kind": "FileChooser",
      "paths": paths || [],
      "choices": choices || [],
      "current_filter": currentFilter || null
    })
  }

  function acceptShare(selectionLine, allowToken) {
    finish({
      "kind": "Share",
      "selection": String(selectionLine || ""),
      "allowToken": allowToken === true
    })
  }

  function acceptConfirm() {
    finish({ "kind": "Confirm", "accepted": true })
  }

  function acceptDynamicLauncher(name) {
    finish({
      "kind": "DynamicLauncher",
      "accepted": true,
      "name": String(name || "")
    })
  }

  function acceptBackground(result) {
    finish({ "kind": "Background", "result": Number(result) })
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

  // Portal dialogs are a normal floating window, not a layer-shell
  // overlay. Match that: a centered floating XDG toplevel sized to the card.
  FloatingWindow {
    id: panel
    visible: root.opened
    // Use a distinct window title when the dialog provides one.
    title: root.kind === "Account" ? "User Information Requested"
         : (root.kind === "Background" ? "Background Activity"
         : (root.kind === "DynamicLauncher" ? "Launcher Requested" : "Omarchy Portal"))
    color: Color.popups.background
    implicitWidth: dialogLoader.item && dialogLoader.item.cardWidth ? dialogLoader.item.cardWidth : 480
    implicitHeight: dialogLoader.item && dialogLoader.item.cardHeight ? dialogLoader.item.cardHeight : 280
    minimumSize: Qt.size(320, 180)

    Rectangle {
      anchors.fill: parent
      color: Color.popups.background
    }

    onVisibleChanged: {
      if (!visible && root.opened && !root.finishing) {
        // Dismissing the Background prompt without a choice is Allow once.
        if (root.kind === "Background")
          root.finish({ "kind": "Background", "result": 2 })
        else
          root.finish({ "kind": "Cancel" })
      }
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
          case "Background": return backgroundComp
          case "Wallpaper": return wallpaperComp
          case "AppChooser": return appComp
          case "FileChooser": return fileComp
          case "Share": return shareComp
          case "Confirm": return confirmComp
          case "DynamicLauncher": return dynamicLauncherComp
          case "Screenshot": return screenshotComp
          default: return accessComp
          }
        }
      }
    }
  }

  Component {
    id: accessComp
    AccessDialog {
      request: root.request
      onDecided: function(granted, choices) {
        if (granted)
          root.acceptAccess(choices)
        else
          root.finish({ "kind": "Cancel" })
      }
    }
  }

  Component {
    id: accountComp
    AccountDialog {
      request: root.request
      extra: root.extra
      onDecided: function(shared) {
        if (shared)
          root.acceptAccount()
        else
          root.finish({ "kind": "Cancel" })
      }
    }
  }

  Component {
    id: backgroundComp
    BackgroundDialog {
      request: root.request
      onDecided: function(result) { root.acceptBackground(result) }
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
      onChosen: function(choice, remember) { root.acceptApp(choice, remember) }
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
    id: dynamicLauncherComp
    DynamicLauncherDialog {
      request: root.request
      extra: root.extra
      onDecided: function(accepted, name) {
        if (accepted)
          root.acceptDynamicLauncher(name)
        else
          root.finish({ "kind": "Cancel" })
      }
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
      onPicked: function(line, allowToken) { root.acceptShare(line, allowToken) }
      onRejected: root.finish({ "kind": "Cancel" })
    }
  }
}
