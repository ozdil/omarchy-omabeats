import QtQuick
import QtQuick.Layouts
import "../theme"

Rectangle {
    id: root

    property string currentMode: "off"
    signal modeSelected(string mode)

    implicitWidth: 320
    implicitHeight: 64
    radius: Theme.radiusMd
    color: Theme.bgCard
    border.color: Theme.border
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            Text {
                text: "󰿎"
                font.family: Theme.iconFont
                font.pixelSize: 13
                color: Theme.accent
            }

            Text {
                text: "Spatial Audio (Uzamsal Ses)"
                font.family: Theme.fontFamily
                font.pixelSize: 11
                font.weight: Font.DemiBold
                color: Theme.textMain
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            readonly property var modes: [
                { id: "off", label: "Kapali" },
                { id: "cinema", label: "Cinema Dolby" },
                { id: "music", label: "Music Stage" }
            ]

            Repeater {
                model: parent.modes

                delegate: Rectangle {
                    id: btn
                    Layout.fillWidth: true
                    implicitHeight: 24
                    radius: Theme.radiusSm
                    color: root.currentMode === modelData.id ? Theme.accent : (btnMouse.containsMouse ? Theme.bgCardHover : Theme.bgDark)

                    Text {
                        anchors.centerIn: parent
                        text: modelData.label
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        font.weight: root.currentMode === modelData.id ? Font.Bold : Font.Normal
                        color: root.currentMode === modelData.id ? "#ffffff" : Theme.textMuted
                    }

                    MouseArea {
                        id: btnMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.modeSelected(modelData.id)
                    }
                }
            }
        }
    }
}
