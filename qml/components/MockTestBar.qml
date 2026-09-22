import QtQuick
import QtQuick.Layouts
import "../theme"

Rectangle {
    id: root

    property bool testModeActive: false
    signal setModel(string modelName)
    signal setBattery(string target, int level)
    signal toggleCharging(string target)
    signal toggleEar(string target)

    implicitWidth: 320
    implicitHeight: 90
    radius: Theme.radiusMd
    color: Theme.bgDark
    border.color: Theme.accentYellow
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            Text {
                text: Theme.iconTestMode
                font.family: Theme.iconFont
                font.pixelSize: 12
                color: Theme.accentYellow
            }

            Text {
                text: "Yerel Test & Simulator Modu"
                font.family: Theme.fontFamily
                font.pixelSize: 10
                font.weight: Font.Bold
                color: Theme.accentYellow
            }

            Item { Layout.fillWidth: true }

            Text {
                text: "Canli Deneme"
                font.family: Theme.fontFamily
                font.pixelSize: 9
                color: Theme.textDim
            }
        }

        // Row 1: Model Switcher
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Repeater {
                model: [
                    { id: "beats_fit_pro", label: "Fit Pro" },
                    { id: "beats_studio_pro", label: "Studio Pro" },
                    { id: "beats_solo_4", label: "Solo 4" },
                    { id: "beats_studio_buds_plus", label: "Buds+" }
                ]

                delegate: Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 20
                    radius: Theme.radiusSm
                    color: modelMouse.containsMouse ? Theme.bgCardHover : Theme.bgCard

                    Text {
                        anchors.centerIn: parent
                        text: modelData.label
                        font.family: Theme.fontFamily
                        font.pixelSize: 9
                        color: Theme.textMain
                    }

                    MouseArea {
                        id: modelMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.setModel(modelData.id)
                    }
                }
            }
        }

        // Row 2: Battery & Ear test triggers
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 20
                radius: Theme.radiusSm
                color: Theme.bgCard
                Text {
                    anchors.centerIn: parent
                    text: "Pil: %15"
                    font.family: Theme.fontFamily
                    font.pixelSize: 9
                    color: Theme.accent
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.setBattery("both", 15)
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 20
                radius: Theme.radiusSm
                color: Theme.bgCard
                Text {
                    anchors.centerIn: parent
                    text: "Pil: %85"
                    font.family: Theme.fontFamily
                    font.pixelSize: 9
                    color: Theme.accentGreen
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.setBattery("both", 85)
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 20
                radius: Theme.radiusSm
                color: Theme.bgCard
                Text {
                    anchors.centerIn: parent
                    text: "Sol Cikar"
                    font.family: Theme.fontFamily
                    font.pixelSize: 9
                    color: Theme.textMuted
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.toggleEar("left")
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 20
                radius: Theme.radiusSm
                color: Theme.bgCard
                Text {
                    anchors.centerIn: parent
                    text: "Sarj Tetikle"
                    font.family: Theme.fontFamily
                    font.pixelSize: 9
                    color: Theme.accentYellow
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.toggleCharging("left")
                }
            }
        }
    }
}
