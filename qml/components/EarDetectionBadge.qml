import QtQuick
import QtQuick.Layouts
import "../theme"

Rectangle {
    id: root

    property bool inEarLeft: true
    property bool inEarRight: true
    property bool autoPauseEnabled: true
    signal autoPauseToggled(bool enabled)

    implicitWidth: 320
    implicitHeight: 46
    radius: Theme.radiusMd
    color: Theme.bgCard
    border.color: Theme.border
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 12
        spacing: 12

        // Left earbud in-ear indicator
        RowLayout {
            spacing: 5
            Rectangle {
                width: 7
                height: 7
                radius: 3.5
                color: root.inEarLeft ? Theme.accentGreen : Theme.textDim
            }
            Text {
                text: "Sol: " + (root.inEarLeft ? "Kulakta" : "Disarida")
                font.family: Theme.fontFamily
                font.pixelSize: 11
                color: root.inEarLeft ? Theme.textMain : Theme.textMuted
            }
        }

        // Right earbud in-ear indicator
        RowLayout {
            spacing: 5
            Rectangle {
                width: 7
                height: 7
                radius: 3.5
                color: root.inEarRight ? Theme.accentGreen : Theme.textDim
            }
            Text {
                text: "Sag: " + (root.inEarRight ? "Kulakta" : "Disarida")
                font.family: Theme.fontFamily
                font.pixelSize: 11
                color: root.inEarRight ? Theme.textMain : Theme.textMuted
            }
        }

        Item { Layout.fillWidth: true }

        // Auto pause button
        Rectangle {
            implicitWidth: 84
            implicitHeight: 26
            radius: Theme.radiusSm
            color: root.autoPauseEnabled ? Theme.bgCardActive : Theme.bgDark
            border.color: root.autoPauseEnabled ? Theme.accentGreen : Theme.border
            border.width: 1

            RowLayout {
                anchors.centerIn: parent
                spacing: 4

                Text {
                    text: root.autoPauseEnabled ? Theme.iconCheck : ""
                    font.family: Theme.iconFont
                    font.pixelSize: 10
                    color: Theme.accentGreen
                    visible: root.autoPauseEnabled
                }

                Text {
                    text: "Oto-Duraklat"
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.weight: root.autoPauseEnabled ? Font.Bold : Font.Normal
                    color: root.autoPauseEnabled ? Theme.textMain : Theme.textMuted
                }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.autoPauseToggled(!root.autoPauseEnabled)
            }
        }
    }
}
