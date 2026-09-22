import QtQuick
import QtQuick.Layouts
import "../theme"

Rectangle {
    id: root

    property string currentMode: "NoiseCancellation"
    property bool hasAdaptive: false
    signal modeChanged(string newMode)

    implicitWidth: 320
    implicitHeight: 44
    radius: Theme.radiusMd
    color: Theme.bgDark
    border.color: Theme.border
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.margins: 3
        spacing: 3

        // Noise Cancellation Button
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Theme.radiusSm
            color: root.currentMode === "NoiseCancellation" ? Theme.bgCardActive : (noiseMouse.containsMouse ? Theme.bgCardHover : "transparent")
            border.color: root.currentMode === "NoiseCancellation" ? Theme.accent : "transparent"
            border.width: root.currentMode === "NoiseCancellation" ? 1 : 0

            RowLayout {
                anchors.centerIn: parent
                spacing: 6

                Text {
                    text: Theme.iconAncNoise
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: root.currentMode === "NoiseCancellation" ? Theme.accent : Theme.textMuted
                }

                Text {
                    text: "ANC"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    font.weight: root.currentMode === "NoiseCancellation" ? Font.Bold : Font.Normal
                    color: root.currentMode === "NoiseCancellation" ? Theme.textMain : Theme.textMuted
                }
            }

            MouseArea {
                id: noiseMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.modeChanged("noise")
            }
        }

        // Off Button
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Theme.radiusSm
            color: root.currentMode === "Off" ? Theme.bgCardActive : (offMouse.containsMouse ? Theme.bgCardHover : "transparent")
            border.color: root.currentMode === "Off" ? Theme.accent : "transparent"
            border.width: root.currentMode === "Off" ? 1 : 0

            RowLayout {
                anchors.centerIn: parent
                spacing: 6

                Text {
                    text: Theme.iconAncOff
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: root.currentMode === "Off" ? Theme.accent : Theme.textMuted
                }

                Text {
                    text: "Kapali"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    font.weight: root.currentMode === "Off" ? Font.Bold : Font.Normal
                    color: root.currentMode === "Off" ? Theme.textMain : Theme.textMuted
                }
            }

            MouseArea {
                id: offMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.modeChanged("off")
            }
        }

        // Transparency Button
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Theme.radiusSm
            color: root.currentMode === "Transparency" ? Theme.bgCardActive : (transpMouse.containsMouse ? Theme.bgCardHover : "transparent")
            border.color: root.currentMode === "Transparency" ? Theme.accent : "transparent"
            border.width: root.currentMode === "Transparency" ? 1 : 0

            RowLayout {
                anchors.centerIn: parent
                spacing: 6

                Text {
                    text: Theme.iconAncTransparency
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: root.currentMode === "Transparency" ? Theme.accent : Theme.textMuted
                }

                Text {
                    text: "Seffaf"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    font.weight: root.currentMode === "Transparency" ? Font.Bold : Font.Normal
                    color: root.currentMode === "Transparency" ? Theme.textMain : Theme.textMuted
                }
            }

            MouseArea {
                id: transpMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.modeChanged("transparency")
            }
        }

        // Adaptive Button (if model supports it)
        Rectangle {
            visible: root.hasAdaptive
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Theme.radiusSm
            color: root.currentMode === "Adaptive" ? Theme.bgCardActive : (adaptMouse.containsMouse ? Theme.bgCardHover : "transparent")
            border.color: root.currentMode === "Adaptive" ? Theme.accent : "transparent"
            border.width: root.currentMode === "Adaptive" ? 1 : 0

            RowLayout {
                anchors.centerIn: parent
                spacing: 6

                Text {
                    text: Theme.iconAncAdaptive
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: root.currentMode === "Adaptive" ? Theme.accent : Theme.textMuted
                }

                Text {
                    text: "Uyumlu"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    font.weight: root.currentMode === "Adaptive" ? Font.Bold : Font.Normal
                    color: root.currentMode === "Adaptive" ? Theme.textMain : Theme.textMuted
                }
            }

            MouseArea {
                id: adaptMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.modeChanged("adaptive")
            }
        }
    }
}
