import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

Panel {
  id: root
  moduleName: "ozdil.omabeats"
  ipcTarget: "ozdil.omabeats"
  manageIpc: true

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  // Beats Headphone Hardware State
  property bool connected: true
  property string modelName: "Beats Fit Pro"
  property string modelId: "beats_fit_pro"
  property string formFactor: "Earbuds"
  property bool hasTriBattery: true
  property bool hasAnc: true
  property bool hasTransparency: true
  property bool hasAdaptive: true
  property bool hasInEar: true
  property bool hasChime: true

  property int batteryLeft: 85
  property bool chargingLeft: false
  property int batteryRight: 80
  property bool chargingRight: false
  property int batteryCase: 95
  property bool chargingCase: true
  property int batterySingle: -1
  property bool chargingSingle: false

  property bool inEarLeft: true
  property bool inEarRight: true
  property string ancMode: "NoiseCancellation"
  property string micMode: "Auto"
  property bool autoPauseEnabled: true
  property string eqProfile: "Beats Signature"
  property string codec: "AAC"
  property int rssi: -54
  property string mac: "04:9D:05:DD:08:62"

  // Unified Single-Color Theme Palette
  readonly property color foreground: bar ? bar.foreground : Color.foreground
  readonly property color dim: Qt.darker(foreground, 1.45)
  readonly property color accent: Color.accent
  readonly property string fontFamily: (bar && bar.fontFamily) ? bar.fontFamily : ((typeof Style !== "undefined" && Style.font && Style.font.family) ? Style.font.family : "JetBrainsMono Nerd Font, JetBrains Mono, monospace")

  function resolveEnginePath() {
    return Qt.resolvedUrl("omabeats-engine").toString().replace(/^file:\/\//, "")
  }

  function resolveCtlPath() {
    return Qt.resolvedUrl("omabeats-ctl").toString().replace(/^file:\/\//, "")
  }

  function runEngineCommand(args) {
    if (!args || args.length === 0) return
    ctlProc.command = [root.resolveCtlPath()].concat(args)
    ctlProc.running = true
  }

  function setEqProfile(name) {
    root.eqProfile = name
    root.runEngineCommand(["set", "eq", name])
  }

  function refresh() {
    if (!statusProc.running) {
      statusProc.running = true
    }
  }

  Process {
    id: statusProc
    command: [root.resolveEnginePath(), "status", "--json"]
    stdout: StdioCollector {
      waitForEnd: true
      onStreamFinished: {
        try {
          var cleanText = String(text || "").slice(0, 65536)
          var d = JSON.parse(cleanText)
          root.connected = !!d.connected
          var m = d.model || {}
          root.modelName = String(m.display_name || d.model_name || "Beats Kulaklik")
          root.modelId = String(m.model_id || d.model_id || "unknown")
          root.formFactor = String(m.form_factor || d.form_factor || "Earbuds")
          root.hasTriBattery = m.has_tri_battery !== undefined ? !!m.has_tri_battery : !!d.has_tri_battery
          root.hasAnc = m.has_anc !== undefined ? !!m.has_anc : !!d.has_anc
          root.hasTransparency = m.has_transparency !== undefined ? !!m.has_transparency : !!d.has_transparency
          root.hasAdaptive = m.has_adaptive !== undefined ? !!m.has_adaptive : !!d.has_adaptive
          root.hasInEar = m.has_in_ear !== undefined ? !!m.has_in_ear : !!d.has_in_ear
          root.hasChime = m.has_chime !== undefined ? !!m.has_chime : !!d.has_chime

          var b = d.battery || {}
          root.batteryLeft = d.battery_left !== undefined ? Number(d.battery_left) : (b.left !== undefined ? Number(b.left) : -1)
          root.chargingLeft = d.charging_left !== undefined ? !!d.charging_left : !!b.charging_left
          root.batteryRight = d.battery_right !== undefined ? Number(d.battery_right) : (b.right !== undefined ? Number(b.right) : -1)
          root.chargingRight = d.charging_right !== undefined ? !!d.charging_right : !!b.charging_right
          root.batteryCase = d.battery_case !== undefined ? Number(d.battery_case) : (b.case !== undefined ? Number(b.case) : -1)
          root.chargingCase = d.charging_case !== undefined ? !!d.charging_case : !!b.charging_case
          root.batterySingle = d.battery_single !== undefined ? Number(d.battery_single) : (b.single !== undefined ? Number(b.single) : -1)
          root.chargingSingle = d.charging_single !== undefined ? !!d.charging_single : !!b.charging_single

          var ie = d.in_ear || {}
          root.inEarLeft = d.in_ear_left !== undefined ? !!d.in_ear_left : !!ie.left
          root.inEarRight = d.in_ear_right !== undefined ? !!d.in_ear_right : !!ie.right

          if (d.anc_mode) root.ancMode = String(d.anc_mode)
          if (d.eq_profile) root.eqProfile = String(d.eq_profile)
          if (d.codec) root.codec = String(d.codec)
          if (d.rssi !== undefined) root.rssi = Number(d.rssi)
          if (d.mac) root.mac = String(d.mac)
          if (d.auto_pause_enabled !== undefined) root.autoPauseEnabled = !!d.auto_pause_enabled
        } catch (e) {
          // ignore parsing error
        }
      }
    }
  }

  Process {
    id: ctlProc
    stdout: StdioCollector {
      waitForEnd: true
      onStreamFinished: {
        root.refresh()
      }
    }
  }

  Timer {
    id: pollTimer
    interval: 3000
    running: true
    repeat: true
    onTriggered: root.refresh()
  }

  // --- Top Bar Icon Button ---
  BarIconButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: "󰋋"
    fontFamily: root.fontFamily
    foreground: bar ? bar.foreground : root.foreground
    tooltipText: root.connected
                 ? ("OmaBeats: " + root.modelName + (root.batteryLeft >= 0 ? (" (" + root.batteryLeft + "%)") : " (Bagli)"))
                 : "OmaBeats: Beats Bagli Degil"
    onPressed: function(b) {
      if (root.opened) root.close()
      else root.open()
    }
  }

  // --- Keyboard Panel Popover Menu ---
  KeyboardPanel {
    id: panel
    anchorItem: button
    owner: root
    bar: root.bar
    open: root.opened
    focusTarget: keyCatcher
    contentWidth: panel.fittedContentWidth(Style.space(380))
    contentHeight: panel.fittedContentHeight(panelColumn.implicitHeight)

    PanelKeyCatcher {
      id: keyCatcher
      anchors.fill: parent
      onCloseRequested: root.close()
      onTabRequested: function(direction) { root.switchPanel(direction) }

      Flickable {
        id: panelFlick
        anchors.fill: parent
        contentWidth: width
        contentHeight: panelColumn.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.VerticalFlick
        interactive: contentHeight > height
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        Column {
          id: panelColumn
          width: panelFlick.width
          spacing: Style.space(10)

          // 1. Hero Header
          Item {
            width: parent.width
            implicitHeight: hero.implicitHeight

            PanelHero {
              id: hero
              width: parent.width
              title: root.modelName
              meta: root.connected ? ("BAGLI · " + root.codec + (root.rssi !== 0 ? (" · " + root.rssi + " dBm") : "")) : "BAGLANTI YOK"
              foreground: root.foreground
              fontFamily: root.fontFamily
              iconComponent: Component {
                Text {
                  text: "󰋋"
                  color: root.foreground
                  font.family: root.fontFamily
                  font.pixelSize: Style.font.display
                }
              }
              trailingControl: Component {
                Button {
                  text: root.connected ? "Kes" : "Baglan"
                  iconText: root.connected ? "󰂲" : "󰂯"
                  bordered: true
                  foreground: root.foreground
                  accent: root.accent
                  fontFamily: root.fontFamily
                  fontSize: Style.font.caption
                  onClicked: {
                    if (root.connected) root.runEngineCommand(["disconnect"])
                    else root.runEngineCommand(["connect"])
                  }
                }
              }
            }
          }

          // 2. Battery Status Section
          PanelSeparator { foreground: root.foreground }

          PanelSectionHeader {
            text: "PIL DURUMU"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          // Tri-Battery Cards (Left, Right, Case)
          Row {
            width: parent.width
            spacing: Style.space(8)
            visible: root.hasTriBattery

            // Left Earbud
            BorderSurface {
              width: Math.floor((parent.width - Style.space(16)) / 3)
              implicitHeight: Style.space(56)
              color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.04)
              radius: Style.cornerRadius
              borderSpec: Border.flat(Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12), 1)

              Column {
                anchors.centerIn: parent
                spacing: Style.space(2)

                Row {
                  anchors.horizontalCenter: parent.horizontalCenter
                  spacing: Style.space(4)
                  Text {
                    text: root.chargingLeft ? "󱐋" : "󰥈"
                    color: root.dim
                    font.family: root.fontFamily
                    font.pixelSize: Style.font.caption
                    anchors.verticalCenter: parent.verticalCenter
                  }
                  Text {
                    text: "SOL"
                    color: root.dim
                    font.family: root.fontFamily
                    font.pixelSize: Style.font.caption
                    font.bold: true
                    anchors.verticalCenter: parent.verticalCenter
                  }
                }

                Text {
                  anchors.horizontalCenter: parent.horizontalCenter
                  text: root.batteryLeft >= 0 ? (root.batteryLeft + "%") : "--"
                  color: root.foreground
                  font.family: root.fontFamily
                  font.pixelSize: Style.font.title
                  font.bold: true
                }
              }
            }

            // Right Earbud
            BorderSurface {
              width: Math.floor((parent.width - Style.space(16)) / 3)
              implicitHeight: Style.space(56)
              color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.04)
              radius: Style.cornerRadius
              borderSpec: Border.flat(Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12), 1)

              Column {
                anchors.centerIn: parent
                spacing: Style.space(2)

                Row {
                  anchors.horizontalCenter: parent.horizontalCenter
                  spacing: Style.space(4)
                  Text {
                    text: root.chargingRight ? "󱐋" : "󰥈"
                    color: root.dim
                    font.family: root.fontFamily
                    font.pixelSize: Style.font.caption
                    anchors.verticalCenter: parent.verticalCenter
                  }
                  Text {
                    text: "SAG"
                    color: root.dim
                    font.family: root.fontFamily
                    font.pixelSize: Style.font.caption
                    font.bold: true
                    anchors.verticalCenter: parent.verticalCenter
                  }
                }

                Text {
                  anchors.horizontalCenter: parent.horizontalCenter
                  text: root.batteryRight >= 0 ? (root.batteryRight + "%") : "--"
                  color: root.foreground
                  font.family: root.fontFamily
                  font.pixelSize: Style.font.title
                  font.bold: true
                }
              }
            }

            // Charging Case
            BorderSurface {
              width: Math.floor((parent.width - Style.space(16)) / 3)
              implicitHeight: Style.space(56)
              color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.04)
              radius: Style.cornerRadius
              borderSpec: Border.flat(Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12), 1)

              Column {
                anchors.centerIn: parent
                spacing: Style.space(2)

                Row {
                  anchors.horizontalCenter: parent.horizontalCenter
                  spacing: Style.space(4)
                  Text {
                    text: root.chargingCase ? "󱐋" : "󰟀"
                    color: root.dim
                    font.family: root.fontFamily
                    font.pixelSize: Style.font.caption
                    anchors.verticalCenter: parent.verticalCenter
                  }
                  Text {
                    text: "KUTU"
                    color: root.dim
                    font.family: root.fontFamily
                    font.pixelSize: Style.font.caption
                    font.bold: true
                    anchors.verticalCenter: parent.verticalCenter
                  }
                }

                Text {
                  anchors.horizontalCenter: parent.horizontalCenter
                  text: root.batteryCase >= 0 ? (root.batteryCase + "%") : "--"
                  color: root.foreground
                  font.family: root.fontFamily
                  font.pixelSize: Style.font.title
                  font.bold: true
                }
              }
            }
          }

          // Single Battery Card (for Over-Ear Headphones)
          BorderSurface {
            width: parent.width
            implicitHeight: Style.space(52)
            visible: !root.hasTriBattery
            color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.04)
            radius: Style.cornerRadius
            borderSpec: Border.flat(Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12), 1)

            Row {
              anchors.fill: parent
              anchors.leftMargin: Style.space(14)
              anchors.rightMargin: Style.space(14)
              spacing: Style.space(10)

              Text {
                text: root.chargingSingle ? "󱐋" : "󰋋"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.subtitle
                anchors.verticalCenter: parent.verticalCenter
              }

              Text {
                text: "Kulaklik Pili"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.body
                font.bold: true
                anchors.verticalCenter: parent.verticalCenter
              }

              Item {
                width: Math.max(0, parent.width - parent.children[0].width - parent.children[1].width - parent.children[3].width - Style.space(30))
                height: 1
              }

              Text {
                text: root.batterySingle >= 0 ? (root.batterySingle + "%") : "--"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.title
                font.bold: true
                anchors.verticalCenter: parent.verticalCenter
              }
            }
          }

          // 3. Noise Cancellation (ANC Modes)
          PanelSeparator {
            foreground: root.foreground
            visible: root.hasAnc
          }

          PanelSectionHeader {
            text: "GURULTU DENETIMI"
            foreground: root.foreground
            fontFamily: root.fontFamily
            visible: root.hasAnc
          }

          Row {
            width: parent.width
            spacing: Style.space(6)
            visible: root.hasAnc

            readonly property var modes: {
              var list = [
                { mode: "NoiseCancellation", label: "ANC", icon: "󰂚" },
                { mode: "Off", label: "Kapali", icon: "󰂛" },
                { mode: "Transparency", label: "Seffaf", icon: "󰂚" }
              ]
              if (root.hasAdaptive) {
                list.push({ mode: "Adaptive", label: "Uyumlu", icon: "󰥒" })
              }
              return list
            }

            Repeater {
              model: parent.modes

              delegate: Button {
                required property var modelData
                width: Math.floor((parent.width - (parent.modes.length - 1) * Style.space(6)) / parent.modes.length)
                text: modelData.label
                iconText: modelData.icon
                bordered: true
                selected: root.ancMode === modelData.mode
                foreground: root.foreground
                accent: root.accent
                fontFamily: root.fontFamily
                fontSize: Style.font.caption
                onClicked: root.runEngineCommand(["anc", modelData.mode.toLowerCase()])
              }
            }
          }

          // 4. In-Ear Detection & Auto-Pause Toggle
          PanelSeparator {
            foreground: root.foreground
            visible: root.hasInEar
          }

          Toggle {
            width: parent.width
            visible: root.hasInEar
            label: "Otomatik Duraklatma"
            description: (root.inEarLeft && root.inEarRight)
                         ? "Kulak ici algilama aktif (Her iki kulaklik kulakta)"
                         : ((root.inEarLeft || root.inEarRight) ? "Kulak ici algilama aktif (Bir kulaklik kulakta)" : "Kulakliklar kulakta degil")
            checked: root.autoPauseEnabled
            foreground: root.foreground
            accent: root.accent
            fontFamily: root.fontFamily
            onClicked: {
              root.autoPauseEnabled = !root.autoPauseEnabled
              root.runEngineCommand(["set", "auto_pause", root.autoPauseEnabled ? "true" : "false"])
            }
          }

          // 5. Sound Profile / Equalizer
          PanelSeparator {
            foreground: root.foreground
          }

          PanelSectionHeader {
            text: "SES PROFILI (EKOLAYZER)"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          Row {
            width: parent.width
            spacing: Style.space(6)

            readonly property var eqList: [
              { id: "Beats Signature", label: "Imza" },
              { id: "Bass Boost", label: "Bas+" },
              { id: "Vocal Clarity", label: "Vokal" },
              { id: "Flat", label: "Flat" }
            ]

            Repeater {
              model: parent.eqList

              delegate: Button {
                required property var modelData
                width: Math.floor((parent.width - 3 * Style.space(6)) / 4)
                text: modelData.label
                bordered: true
                selected: root.eqProfile === modelData.id
                foreground: root.foreground
                accent: root.accent
                fontFamily: root.fontFamily
                fontSize: Style.font.caption
                onClicked: root.setEqProfile(modelData.id)
              }
            }
          }

          // 6. Find My / Chime Actions
          PanelSeparator {
            foreground: root.foreground
            visible: root.hasChime
          }

          PanelSectionHeader {
            text: "KULAKLIKLARI BUL (SES CALDIR)"
            foreground: root.foreground
            fontFamily: root.fontFamily
            visible: root.hasChime
          }

          Row {
            width: parent.width
            spacing: Style.space(8)
            visible: root.hasChime

            Button {
              width: Math.floor((parent.width - Style.space(8)) / 2)
              text: "Sol Caldir"
              iconText: "󰂞"
              bordered: true
              foreground: root.foreground
              accent: root.accent
              fontFamily: root.fontFamily
              fontSize: Style.font.caption
              onClicked: root.runEngineCommand(["chime", "left"])
            }

            Button {
              width: Math.floor((parent.width - Style.space(8)) / 2)
              text: "Sag Caldir"
              iconText: "󰂞"
              bordered: true
              foreground: root.foreground
              accent: root.accent
              fontFamily: root.fontFamily
              fontSize: Style.font.caption
              onClicked: root.runEngineCommand(["chime", "right"])
            }
          }

          // 7. Footer Info & Refresh
          PanelSeparator {
            foreground: root.foreground
          }

          Item {
            width: parent.width
            implicitHeight: refreshBtn.implicitHeight

            Row {
              anchors.left: parent.left
              anchors.verticalCenter: parent.verticalCenter
              spacing: Style.space(6)

              Text {
                text: "MAC: " + root.mac
                color: root.dim
                font.family: root.fontFamily
                font.pixelSize: Style.font.caption
                anchors.verticalCenter: parent.verticalCenter
              }
            }

            Button {
              id: refreshBtn
              anchors.right: parent.right
              anchors.verticalCenter: parent.verticalCenter
              text: "Yenile"
              iconText: "󰑐"
              bordered: true
              foreground: root.foreground
              accent: root.accent
              fontFamily: root.fontFamily
              fontSize: Style.font.caption
              onClicked: root.refresh()
            }
          }
        }
      }
    }
  }
}
