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
  property bool connected: false
  property bool showAboutModal: false
  property string modelName: "Beats Fit Pro"
  property string modelId: "beats_fit_pro"
  property string formFactor: "Earbuds"
  property bool hasTriBattery: true
  property bool hasAnc: true
  property bool hasTransparency: true
  property bool hasAdaptive: true
  property bool hasInEar: true
  property bool hasChime: true
  property bool isWired: false
  property string connectionType: ""
  property string wiredModel: ""
  readonly property bool isBothCharging: (root.chargingLeft && root.chargingRight && !root.inEarLeft && !root.inEarRight)

  property int batteryLeft: -1
  property bool chargingLeft: false
  property int batteryRight: -1
  property bool chargingRight: false
  property int batteryCase: -1
  property bool chargingCase: false
  property int batterySingle: -1
  property bool chargingSingle: false

  property int volume: 70
  property bool inEarLeft: true
  property bool inEarRight: true
  property string ancMode: "NoiseCancellation"
  property int noiseLevel: 0
  property string micMode: "Auto"
  property bool autoPauseEnabled: true
  property string codec: "AAC"
  property string spatialMode: "off"
  property string eqProfile: "Flat"
  property int rssi: -60
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
          root.modelName = String(m.display_name || d.model_name || "Beats Headphones")
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

          if (d.volume !== undefined) root.volume = Number(d.volume)

          var ie = d.in_ear || {}
          root.inEarLeft = d.in_ear_left !== undefined ? !!d.in_ear_left : !!ie.left
          root.inEarRight = d.in_ear_right !== undefined ? !!d.in_ear_right : !!ie.right

          if (d.anc_mode) root.ancMode = String(d.anc_mode)
          if (d.noise_control_level !== undefined) {
            root.noiseLevel = Number(d.noise_control_level)
          } else {
            var mLower = String(root.ancMode).toLowerCase()
            root.noiseLevel = (mLower.indexOf("noise") !== -1 || mLower.indexOf("anc") !== -1) ? 0 : (mLower.indexOf("off") !== -1 ? 50 : 100)
          }
          if (d.codec) root.codec = String(d.codec)
          if (d.spatial_audio_mode) root.spatialMode = String(d.spatial_audio_mode)
          if (d.eq_profile) root.eqProfile = String(d.eq_profile)
          if (d.rssi !== undefined) root.rssi = Number(d.rssi)
          if (d.mac) root.mac = String(d.mac)
          if (d.auto_pause_enabled !== undefined) root.autoPauseEnabled = !!d.auto_pause_enabled
          root.isWired = !!d.is_wired
          root.connectionType = String(d.connection_type || "")
          root.wiredModel = String(d.wired_model || "")

          if (root.isBothCharging) {
            if (root.opened) root.close()
          }
        } catch (e) {
          // ignore parsing error
        }
      }
    }
  }

  Process {
    id: ctlProc
    onExited: function() {
      root.refresh()
    }
    stdout: StdioCollector {
      waitForEnd: true
      onStreamFinished: {
        root.refresh()
      }
    }
  }

  Timer {
    id: pollTimer
    interval: 2000
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
    opacity: !root.connected ? 0.35 : (root.isBothCharging ? 0.55 : 1.0)
    fontFamily: root.fontFamily
    foreground: bar ? bar.foreground : root.foreground
    tooltipText: root.isBothCharging
                 ? ("OmaBeats: " + root.modelName + " (Charging in Case)")
                 : (root.connected
                    ? ("OmaBeats: " + root.modelName + (root.isWired ? " (Wired)" : (root.batteryLeft >= 0 ? (" (" + root.batteryLeft + "%)") : " (Connected)")))
                    : "OmaBeats: Disconnected")
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
      onCloseRequested: {
        if (root.showAboutModal) {
          root.showAboutModal = false
        } else {
          root.close()
        }
      }
      onTabRequested: function(direction) { root.switchPanel(direction) }
      onTextKey: function(t) {
        if (t === "r" || t === "R") {
          root.refresh()
        } else if (t === "a" || t === "A") {
          root.showAboutModal = !root.showAboutModal
        } else if (t === "1") {
          root.setNoiseControl("anc")
        } else if (t === "2") {
          root.setNoiseControl("off")
        } else if (t === "3") {
          root.setNoiseControl("transparency")
        }
      }

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
              meta: root.isBothCharging
                    ? "CHARGING IN CASE · STANDBY"
                    : (root.connected
                       ? ((root.isWired ? "WIRED · " : "CONNECTED · ") + root.codec + (!root.isWired && root.rssi !== 0 ? (" · " + root.rssi + " dBm") : ""))
                       : "NOT CONNECTED")
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
                  text: root.connected ? "Disconnect" : "Connect"
                  iconText: root.connected ? "󰂲" : "󰂯"
                  bordered: true
                  foreground: root.foreground
                  accent: root.accent
                  fontFamily: root.fontFamily
                  fontSize: Style.font.caption
                  onClicked: {
                    if (root.connected) {
                      root.connected = false
                      root.runEngineCommand(["disconnect"])
                    } else {
                      root.runEngineCommand(["connect"])
                    }
                  }
                }
              }
            }
          }

          // 2. Battery Status Section
          PanelSeparator { foreground: root.foreground }

          PanelSectionHeader {
            text: "BATTERY STATUS"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          // Wired Mode Indicator Card (Lossless USB-C or 3.5mm Analog)
          BorderSurface {
            width: parent.width
            implicitHeight: Style.space(44)
            visible: root.isWired
            color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.04)
            radius: Style.cornerRadius
            borderSpec: Border.flat(Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12), 1)

            Row {
              anchors.fill: parent
              anchors.leftMargin: Style.space(14)
              anchors.rightMargin: Style.space(14)
              spacing: Style.space(10)

              Text {
                text: "󰋋"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.subtitle
                anchors.verticalCenter: parent.verticalCenter
              }

              Text {
                text: root.connectionType !== "" ? root.connectionType : "Wired Connection (Continuous Power)"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.caption
                font.bold: true
                anchors.verticalCenter: parent.verticalCenter
              }
            }
          }

          // Tri-Battery Cards (Left, Right, Case)
          Row {
            width: parent.width
            spacing: Style.space(8)
            visible: root.hasTriBattery && !root.isWired

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
                    text: "LEFT"
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
                    text: "RIGHT"
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
                    text: "CASE"
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
            visible: !root.hasTriBattery && !root.isWired
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
                text: "Headphone Battery"
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

          // 3. Volume Slider Section
          PanelSeparator { foreground: root.foreground }

          PanelSectionHeader {
            text: "VOLUME"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          BorderSurface {
            width: parent.width
            implicitHeight: Style.space(52)
            color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.04)
            radius: Style.cornerRadius
            borderSpec: Border.flat(Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12), 1)

            Row {
              anchors.fill: parent
              anchors.leftMargin: Style.space(12)
              anchors.rightMargin: Style.space(12)
              spacing: Style.space(10)

              Text {
                text: root.volume === 0 ? "󰝟" : (root.volume < 50 ? "󰕿" : "󰕾")
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.subtitle
                anchors.verticalCenter: parent.verticalCenter
              }

              PanelSlider {
                id: volSlider
                width: parent.width - Style.space(90)
                anchors.verticalCenter: parent.verticalCenter
                bar: root.bar
                minimum: 0
                maximum: 100
                integer: true
                step: 1
                value: root.volume
                trackColor: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.15)
                fillColor: root.foreground
                knobColor: root.foreground
                onMoved: function(next) {
                  root.volume = Math.round(next)
                  root.runEngineCommand(["volume", String(root.volume)])
                }
                onReleased: function(next) {
                  root.volume = Math.round(next)
                  root.runEngineCommand(["volume", String(root.volume)])
                }
              }

              Text {
                text: root.volume + "%"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.body
                font.bold: true
                anchors.verticalCenter: parent.verticalCenter
              }
            }
          }

          // 4. Noise Cancellation (ANC Modes)
          PanelSeparator {
            foreground: root.foreground
            visible: root.hasAnc
          }

          PanelSectionHeader {
            text: "NOISE CONTROL"
            foreground: root.foreground
            fontFamily: root.fontFamily
            visible: root.hasAnc
          }

          Column {
            width: parent.width
            spacing: Style.space(8)
            visible: root.hasAnc

            Row {
              width: parent.width
              Text {
                text: root.noiseLevel <= 30
                      ? "ANC Active (Max Noise Cancellation)"
                      : (root.noiseLevel <= 50
                         ? "Off (Passive Isolation)"
                         : ("Transparency (" + Math.round((root.noiseLevel - 50) * 2) + "% Ambient Passthrough)"))
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.caption
                font.bold: true
                anchors.verticalCenter: parent.verticalCenter
              }

              Item { width: Style.space(6); height: 1 }

              Text {
                text: root.noiseLevel + "/100"
                color: root.dim
                font.family: root.fontFamily
                font.pixelSize: Style.font.fineprint
                anchors.verticalCenter: parent.verticalCenter
              }
            }

            PanelSlider {
              id: noiseSlider
              width: parent.width
              bar: root.bar
              minimum: 0
              maximum: 100
              integer: true
              step: 1
              value: root.noiseLevel
              trackColor: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.15)
              fillColor: root.foreground
              knobColor: root.foreground
              onMoved: function(next) {
                root.noiseLevel = Math.round(next)
                root.runEngineCommand(["anc", String(root.noiseLevel)])
              }
              onReleased: function(next) {
                root.noiseLevel = Math.round(next)
                root.runEngineCommand(["anc", String(root.noiseLevel)])
              }
            }

            Row {
              width: parent.width
              spacing: Style.space(6)

              readonly property var presets: [
                { label: "ANC", level: 0, icon: "󰂚" },
                { label: "Off", level: 50, icon: "󰂛" },
                { label: "Transparency", level: 100, icon: "󰂚" }
              ]

              Repeater {
                model: parent.presets

                delegate: Button {
                  required property var modelData
                  width: Math.floor((parent.width - 2 * Style.space(6)) / 3)
                  text: modelData.label
                  iconText: modelData.icon
                  bordered: true
                  horizontalPadding: Style.space(4)
                  selected: (modelData.level === 0 && root.noiseLevel <= 30) ||
                            (modelData.level === 50 && root.noiseLevel > 30 && root.noiseLevel <= 50) ||
                            (modelData.level === 100 && root.noiseLevel > 50)
                  foreground: root.foreground
                  accent: root.accent
                  fontFamily: root.fontFamily
                  fontSize: Style.font.caption
                  onClicked: {
                    root.noiseLevel = modelData.level
                    root.runEngineCommand(["anc", String(modelData.level)])
                  }
                }
              }
            }
          }

          // 5. Spatial Audio (Uzamsal Ses / Dolby Atmos & Apple Spatial Stage)
          PanelSeparator {
            foreground: root.foreground
          }

          PanelSectionHeader {
            text: "SPATIAL AUDIO (UZAMSAL SES)"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          Row {
            width: parent.width
            spacing: Style.space(6)

            readonly property var spatialPresets: [
              { label: "Off", id: "off", icon: "󰝟" },
              { label: "Cinema Dolby", id: "cinema", icon: "󰿎" },
              { label: "Music Stage", id: "music", icon: "󰎆" }
            ]

            Repeater {
              model: parent.spatialPresets

              delegate: Button {
                required property var modelData
                width: Math.floor((parent.width - 2 * Style.space(6)) / 3)
                text: modelData.label
                iconText: modelData.icon
                bordered: true
                horizontalPadding: Style.space(4)
                selected: root.spatialMode === modelData.id
                foreground: root.foreground
                accent: root.accent
                fontFamily: root.fontFamily
                fontSize: Style.font.caption
                onClicked: {
                  root.spatialMode = modelData.id
                  root.runEngineCommand(["spatial", modelData.id])
                }
              }
            }
          }

          // 6. Equalizer Profiles (EQ)
          PanelSeparator {
            foreground: root.foreground
          }

          PanelSectionHeader {
            text: "EQUALIZER PROFILES"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          Flow {
            width: parent.width
            spacing: Style.space(6)

            readonly property var eqPresets: [
              { label: "Beats Signature", id: "Beats Signature" },
              { label: "Bass Boost", id: "Bass Boost" },
              { label: "Vocal Clarity", id: "Vocal Clarity" },
              { label: "Flat", id: "Flat" }
            ]

            Repeater {
              model: parent.eqPresets

              delegate: Button {
                required property var modelData
                text: modelData.label
                bordered: true
                selected: root.eqProfile === modelData.id
                foreground: root.foreground
                accent: root.accent
                fontFamily: root.fontFamily
                fontSize: Style.font.caption
                onClicked: {
                  root.eqProfile = modelData.id
                  root.runEngineCommand(["eq", modelData.id])
                }
              }
            }
          }

          // 7. In-Ear Detection & Auto-Pause Toggle
          PanelSeparator {
            foreground: root.foreground
            visible: root.hasInEar
          }

          Toggle {
            width: parent.width
            visible: root.hasInEar
            label: "Automatic Ear Detection"
            description: (root.inEarLeft && root.inEarRight)
                         ? "In-ear detection active (Both earbuds in ear)"
                         : ((root.inEarLeft || root.inEarRight) ? "In-ear detection active (One earbud in ear)" : "Earbuds out of ear (Media paused)")
            checked: root.autoPauseEnabled
            foreground: root.foreground
            accent: root.accent
            fontFamily: root.fontFamily
            onClicked: {
              root.autoPauseEnabled = !root.autoPauseEnabled
              root.runEngineCommand(["auto-pause", root.autoPauseEnabled ? "true" : "false"])
            }
          }


          // 7. Find My / Chime Actions
          PanelSeparator {
            foreground: root.foreground
            visible: root.hasChime
          }

          PanelSectionHeader {
            text: "FIND MY EARBUDS (PLAY SOUND)"
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
              text: "Ring Left"
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
              text: "Ring Right"
              iconText: "󰂞"
              bordered: true
              foreground: root.foreground
              accent: root.accent
              fontFamily: root.fontFamily
              fontSize: Style.font.caption
              onClicked: root.runEngineCommand(["chime", "right"])
            }
          }

          // Wired Beats Quick Model Selector
          PanelSeparator {
            foreground: root.foreground
          }

          PanelSectionHeader {
            text: "WIRED BEATS PROFILE"
            foreground: root.foreground
            fontFamily: root.fontFamily
          }

          Flow {
            width: parent.width
            spacing: Style.space(6)

            readonly property var wiredOptions: [
              { label: "Beats EP", id: "beats_ep" },
              { label: "Beats Pro", id: "beats_pro" },
              { label: "urBeats 3", id: "urbeats_3" },
              { label: "Solo HD", id: "beats_solo_hd" },
              { label: "Studio Wired", id: "beats_studio_2_wired" },
              { label: "Sifirla", id: "reset" }
            ]

            Repeater {
              model: parent.wiredOptions

              delegate: Button {
                required property var modelData
                text: modelData.label
                bordered: true
                selected: root.wiredModel === modelData.id
                foreground: root.foreground
                accent: root.accent
                fontFamily: root.fontFamily
                fontSize: Style.font.caption
                onClicked: {
                  if (modelData.id === "reset") {
                    root.runEngineCommand(["wired", "reset"])
                  } else {
                    root.runEngineCommand(["wired", modelData.id])
                  }
                }
              }
            }
          }

          // 8. Footer Info & Refresh
          PanelSeparator {
            foreground: root.foreground
          }

          Item {
            width: parent.width
            implicitHeight: refreshBtn.implicitHeight

            Row {
              anchors.left: parent.left
              anchors.verticalCenter: parent.verticalCenter
              spacing: Style.space(8)

              Button {
                iconText: "󰋽"
                tooltipText: "About & Imprint"
                bordered: true
                foreground: root.foreground
                accent: root.accent
                fontFamily: root.fontFamily
                fontSize: Style.font.caption
                onClicked: {
                  root.showAboutModal = !root.showAboutModal
                }
              }

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
              text: "Refresh"
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

      // About & Imprint Modal Overlay
      Rectangle {
        id: aboutOverlay
        anchors.fill: parent
        visible: root.showAboutModal
        color: Qt.rgba(0.05, 0.05, 0.07, 0.96)
        z: 99

        MouseArea {
          anchors.fill: parent
          // Block underlying clicks
        }

        Column {
          anchors.centerIn: parent
          width: parent.width - Style.space(40)
          spacing: Style.space(12)

          Row {
            width: parent.width
            Item {
              width: parent.width - closeAboutBtn.implicitWidth
              implicitHeight: aboutTitleText.implicitHeight
              Text {
                id: aboutTitleText
                text: "OmaBeats"
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.title
                font.bold: true
              }
            }

            Button {
              id: closeAboutBtn
              text: "✕"
              bordered: true
              foreground: root.foreground
              fontFamily: root.fontFamily
              fontSize: Style.font.caption
              onClicked: root.showAboutModal = false
            }
          }

          Text {
            text: "Version: 1.1.0\nDeveloper: Ozan Ozdil (@ozdil)\nLicense: MIT\nApple Beats & W1/H1 Hardware Management Module"
            color: root.dim
            font.family: root.fontFamily
            font.pixelSize: Style.font.caption
            lineHeight: 1.3
          }

          PanelSeparator {
            width: parent.width
            foreground: root.foreground
          }

          Button {
            width: parent.width
            text: "GitHub / Contact"
            iconText: "󰊤"
            bordered: true
            foreground: root.foreground
            accent: root.accent
            fontFamily: root.fontFamily
            fontSize: Style.font.caption
            onClicked: Qt.openUrlExternally("https://github.com/ozdil")
          }

          Button {
            width: parent.width
            text: "Buy Me a Coffee"
            iconText: "󰅖"
            bordered: true
            foreground: "#000000"
            color: "#FFDD00"
            fontFamily: root.fontFamily
            fontSize: Style.font.caption
            onClicked: Qt.openUrlExternally("https://buymeacoffee.com/ozdil")
          }
        }
      }
    }
  }
}
