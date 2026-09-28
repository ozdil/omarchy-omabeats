import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import "theme"
import "components"

Rectangle {
    id: root

    property bool connected: false
    property bool testMode: false
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
    property bool showTestBar: true

    readonly property string enginePath: {
        var p = Qt.resolvedUrl("../omabeats-engine").toString().replace(/^file:\/\//, "");
        if (p.indexOf("qrc:") === 0 || p.length === 0) {
            return "/home/ozdil/Projects/omarchy/omarchy-omabeats/omabeats-engine";
        }
        return p;
    }
    readonly property string stateFilePath: Theme.homeDir + "/.local/state/omarchy/omabeats_state.json"

    color: Theme.bgBase
    border.color: Theme.border
    border.width: 1

    function loadStateFromJson(jsonStr) {
        if (!jsonStr || jsonStr.trim().length === 0) return;
        try {
            var data = JSON.parse(jsonStr);
            root.connected = data.connected || false;
            root.testMode = data.test_mode || false;
            if (data.mac) root.mac = data.mac;
            if (data.codec) root.codec = data.codec;
            if (data.rssi) root.rssi = data.rssi;
            if (data.anc_mode) root.ancMode = data.anc_mode;
            if (data.mic_mode) root.micMode = data.mic_mode;
            if (data.eq_profile) root.eqProfile = data.eq_profile;
            root.autoPauseEnabled = (data.auto_pause_enabled !== false);

            root.batteryLeft = data.battery_left !== undefined ? data.battery_left : -1;
            root.chargingLeft = data.charging_left || false;
            root.batteryRight = data.battery_right !== undefined ? data.battery_right : -1;
            root.chargingRight = data.charging_right || false;
            root.batteryCase = data.battery_case !== undefined ? data.battery_case : -1;
            root.chargingCase = data.charging_case || false;
            root.batterySingle = data.battery_single !== undefined ? data.battery_single : -1;
            root.chargingSingle = data.charging_single || false;

            root.inEarLeft = data.in_ear_left !== false;
            root.inEarRight = data.in_ear_right !== false;

            if (data.model) {
                root.modelName = data.model.display_name || "Beats";
                root.modelId = data.model.model_id || "beats_fit_pro";
                root.formFactor = data.model.form_factor || "Earbuds";
                root.hasTriBattery = (data.model.has_tri_battery !== false);
                root.hasAnc = (data.model.has_anc !== false);
                root.hasTransparency = (data.model.has_transparency !== false);
                root.hasAdaptive = (data.model.has_adaptive || false);
                root.hasInEar = (data.model.has_in_ear || false);
                root.hasChime = (data.model.has_chime !== false);
            }
        } catch(e) {
            // Partial writes ignored
        }
    }

    function runEngineCommand(args) {
        cmdProc.command = [root.enginePath].concat(args);
        cmdProc.running = true;
    }

    Process {
        id: cmdProc
        running: false
        onExited: {
            refreshProc.running = true;
        }
    }

    Process {
        id: refreshProc
        command: [root.enginePath, "status"]
        running: true
        stdout: StdioCollector {
            onDataChanged: {
                root.loadStateFromJson(text);
            }
        }
    }

    Timer {
        interval: 2000
        running: true
        repeat: true
        onTriggered: {
            if (!refreshProc.running) {
                refreshProc.running = true;
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 14
        spacing: 12

        // Header Card
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 60
            radius: Theme.radiusMd
            color: Theme.bgCard
            border.color: Theme.border
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 10

                Rectangle {
                    implicitWidth: 40
                    implicitHeight: 40
                    radius: Theme.radiusSm
                    color: Theme.bgDark

                    Text {
                        anchors.centerIn: parent
                        text: Theme.iconHeadphones
                        font.family: Theme.iconFont
                        font.pixelSize: 18
                        color: root.connected ? Theme.accent : Theme.textDim
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    RowLayout {
                        spacing: 6
                        Text {
                            text: root.modelName
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            font.weight: Font.Bold
                            color: Theme.textMain
                        }

                        Rectangle {
                            implicitWidth: 44
                            implicitHeight: 16
                            radius: 3
                            color: Theme.bgDark
                            Text {
                                anchors.centerIn: parent
                                text: root.codec
                                font.family: Theme.fontFamily
                                font.pixelSize: 9
                                color: Theme.textMuted
                            }
                        }
                    }

                    Text {
                        text: root.connected ? ("Connected (" + root.rssi + " dBm)") : "Not Connected"
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        color: root.connected ? Theme.accentGreen : Theme.textDim
                    }
                }

                Rectangle {
                    implicitWidth: 70
                    implicitHeight: 26
                    radius: Theme.radiusSm
                    color: root.connected ? Theme.bgCardHover : Theme.accent
                    border.color: Theme.border
                    border.width: 1

                    Text {
                        anchors.centerIn: parent
                        text: root.connected ? "Disconnect" : "Connect"
                        font.family: Theme.fontFamily
                        font.pixelSize: 9
                        font.weight: Font.Bold
                        color: root.connected ? Theme.textMain : "#ffffff"
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (root.connected) {
                                root.runEngineCommand(["disconnect"]);
                            } else {
                                root.runEngineCommand(["connect"]);
                            }
                        }
                    }
                }
            }
        }

        // Battery Status Section
        Item {
            Layout.fillWidth: true
            implicitHeight: 74

            RowLayout {
                anchors.fill: parent
                spacing: 8
                visible: root.hasTriBattery

                BatteryGauge {
                    Layout.fillWidth: true
                    label: "Sol"
                    iconText: Theme.iconEarbudLeft
                    batteryLevel: root.batteryLeft
                    isCharging: root.chargingLeft
                }

                BatteryGauge {
                    Layout.fillWidth: true
                    label: "Sag"
                    iconText: Theme.iconEarbudRight
                    batteryLevel: root.batteryRight
                    isCharging: root.chargingRight
                }

                BatteryGauge {
                    Layout.fillWidth: true
                    label: "Kutu"
                    iconText: Theme.iconBatteryCase
                    batteryLevel: root.batteryCase
                    isCharging: root.chargingCase
                }
            }

            BatteryGauge {
                anchors.fill: parent
                visible: !root.hasTriBattery
                label: root.modelName + " Pil"
                iconText: Theme.iconHeadphones
                batteryLevel: root.batterySingle >= 0 ? root.batterySingle : 80
                isCharging: root.chargingSingle
            }
        }

        // ANC Controls
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            visible: root.hasAnc

            Text {
                text: "Gurultu Denetimi"
                font.family: Theme.fontFamily
                font.pixelSize: 11
                font.weight: Font.DemiBold
                color: Theme.textMuted
            }

            AncSelector {
                Layout.fillWidth: true
                currentMode: root.ancMode
                hasAdaptive: root.hasAdaptive
                onModeChanged: function(newMode) {
                    root.runEngineCommand(["anc", newMode]);
                }
            }
        }

        // In-Ear Detection
        EarDetectionBadge {
            Layout.fillWidth: true
            visible: root.hasInEar
            inEarLeft: root.inEarLeft
            inEarRight: root.inEarRight
            autoPauseEnabled: root.autoPauseEnabled
            onAutoPauseToggled: function(enabled) {
                root.runEngineCommand(["set", "auto_pause", enabled ? "true" : "false"]);
            }
        }

        // EQ Profiles
        EqProfileSelector {
            Layout.fillWidth: true
            currentProfile: root.eqProfile
            onProfileSelected: function(prof) {
                root.runEngineCommand(["eq", prof]);
            }
        }

        // Quick Actions
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            visible: root.hasChime

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 30
                radius: Theme.radiusSm
                color: Theme.bgCard
                border.color: Theme.border
                border.width: 1

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 4
                    Text {
                        text: Theme.iconChime
                        font.family: Theme.iconFont
                        font.pixelSize: 11
                        color: Theme.accentYellow
                    }
                    Text {
                        text: "Sol Caldir"
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        color: Theme.textMain
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.runEngineCommand(["chime", "left"])
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 30
                radius: Theme.radiusSm
                color: Theme.bgCard
                border.color: Theme.border
                border.width: 1

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 4
                    Text {
                        text: Theme.iconChime
                        font.family: Theme.iconFont
                        font.pixelSize: 11
                        color: Theme.accentYellow
                    }
                    Text {
                        text: "Sag Caldir"
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        color: Theme.textMain
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.runEngineCommand(["chime", "right"])
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 30
                radius: Theme.radiusSm
                color: Theme.bgCard
                border.color: Theme.border
                border.width: 1

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 4
                    Text {
                        text: Theme.iconRefresh
                        font.family: Theme.iconFont
                        font.pixelSize: 11
                        color: Theme.accentBlue
                    }
                    Text {
                        text: "Refresh"
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        color: Theme.textMain
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.runEngineCommand(["sync"])
                }
            }
        }

        // Interactive Local Test / Mock Toolbar
        MockTestBar {
            Layout.fillWidth: true
            visible: root.showTestBar
            testModeActive: root.testMode
            onSetModel: function(m) {
                root.runEngineCommand(["mock", m]);
            }
            onSetBattery: function(target, lvl) {
                root.runEngineCommand(["set", "bat_left", lvl.toString()]);
                root.runEngineCommand(["set", "bat_right", lvl.toString()]);
            }
            onToggleCharging: function(target) {
                root.runEngineCommand(["set", "charging_left", root.chargingLeft ? "false" : "true"]);
            }
            onToggleEar: function(target) {
                root.runEngineCommand(["set", "ear_left", root.inEarLeft ? "false" : "true"]);
            }
        }

        Item { Layout.fillHeight: true }
    }
}
