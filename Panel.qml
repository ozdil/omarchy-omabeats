import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import "qml/theme"
import "qml/components"

Scope {
    id: root

    // OmaBeats Headphone State
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

    readonly property string enginePath: Qt.resolvedUrl("omabeats-engine").toString().replace(/^file:\/\//, "")
    readonly property string stateFilePath: Theme.homeDir + "/.local/state/omarchy/omabeats_state.json"

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
            // Ignore parse errors on partial writes
        }
    }

    function runEngineCommand(args) {
        cmdProc.command = [root.enginePath].concat(args);
        cmdProc.running = true;
    }

    // Process to run engine actions
    Process {
        id: cmdProc
        running: false
        onExited: {
            refreshProc.running = true;
        }
    }

    // Process to read status JSON
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
        interval: 2500
        running: true
        repeat: true
        onTriggered: {
            if (!refreshProc.running) {
                refreshProc.running = true;
            }
        }
    }

    // --- BAR WIDGET (OMARCHY TOP BAR) ---
    Rectangle {
        id: barBtn
        implicitWidth: barContent.implicitWidth + 14
        implicitHeight: 28
        radius: Theme.radiusSm
        color: btnMouse.containsMouse ? Theme.bgCardHover : "transparent"

        RowLayout {
            id: barContent
            anchors.centerIn: parent
            spacing: 5

            Text {
                text: Theme.iconHeadphones
                font.family: Theme.iconFont
                font.pixelSize: 13
                color: root.connected ? (btnMouse.containsMouse ? Theme.accent : Theme.textMain) : Theme.textDim
            }

            // ANC status indicator badge
            Text {
                visible: root.connected && root.hasAnc
                text: {
                    if (root.ancMode === "NoiseCancellation") return Theme.iconAncNoise;
                    if (root.ancMode === "Transparency") return Theme.iconAncTransparency;
                    if (root.ancMode === "Adaptive") return Theme.iconAncAdaptive;
                    return Theme.iconAncOff;
                }
                font.family: Theme.iconFont
                font.pixelSize: 11
                color: root.ancMode === "NoiseCancellation" ? Theme.accent : (root.ancMode === "Transparency" ? Theme.accentBlue : Theme.textMuted)
            }

            // Battery display in bar
            Text {
                visible: root.connected
                text: {
                    if (root.hasTriBattery && root.batteryLeft >= 0) {
                        return root.batteryLeft + "%";
                    }
                    if (root.batterySingle >= 0) {
                        return root.batterySingle + "%";
                    }
                    return "";
                }
                font.family: Theme.fontFamily
                font.pixelSize: 11
                font.weight: Font.DemiBold
                color: {
                    var lvl = root.hasTriBattery ? root.batteryLeft : root.batterySingle;
                    if (lvl > 40) return Theme.textMain;
                    if (lvl > 15) return Theme.accentYellow;
                    return Theme.accent;
                }
            }
        }

        MouseArea {
            id: btnMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                dashboardWindow.visible = !dashboardWindow.visible;
                if (dashboardWindow.visible) {
                    refreshProc.running = true;
                }
            }
        }
    }

    // --- MAIN DASHBOARD POPUP WINDOW ---
    FloatingWindow {
        id: dashboardWindow
        title: "OmaBeats - Beats Control Hub"
        implicitWidth: 380
        implicitHeight: root.showTestBar ? 640 : 540
        visible: false
        color: Theme.bgBase

        Rectangle {
            anchors.fill: parent
            color: Theme.bgBase
            border.color: Theme.border
            border.width: 1

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

                        // Device icon badge
                        Rectangle {
                            implicitWidth: 38
                            implicitHeight: 38
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
                                text: root.connected ? ("Bagli (" + root.rssi + " dBm)") : "Baglanti Yok"
                                font.family: Theme.fontFamily
                                font.pixelSize: 10
                                color: root.connected ? Theme.accentGreen : Theme.textDim
                            }
                        }

                        // Connect / Disconnect button
                        Rectangle {
                            implicitWidth: 70
                            implicitHeight: 26
                            radius: Theme.radiusSm
                            color: root.connected ? Theme.bgCardHover : Theme.accent
                            border.color: Theme.border
                            border.width: 1

                            Text {
                                anchors.centerIn: parent
                                text: root.connected ? "Baglantiyi Kes" : "Baglan"
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

                    // Single battery for Over-Ear (Studio Pro / Solo 4)
                    BatteryGauge {
                        anchors.fill: parent
                        visible: !root.hasTriBattery
                        label: root.modelName + " Pil"
                        iconText: Theme.iconHeadphones
                        batteryLevel: root.batterySingle >= 0 ? root.batterySingle : 80
                        isCharging: root.chargingSingle
                    }
                }

                // ANC Controls (if device supports ANC)
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

                // In-Ear Detection (if earbuds)
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

                // Quick Actions: Find My / Chime
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
                                text: "Yenile"
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
    }
}
