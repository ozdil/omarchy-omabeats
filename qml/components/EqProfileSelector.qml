import QtQuick
import QtQuick.Layouts
import "../theme"

Rectangle {
    id: root

    property string currentProfile: "Beats Signature"
    signal profileSelected(string profile)

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
                text: Theme.iconWaveform
                font.family: Theme.iconFont
                font.pixelSize: 13
                color: Theme.accent
            }

            Text {
                text: "Ses Profili / EQ"
                font.family: Theme.fontFamily
                font.pixelSize: 11
                font.weight: Font.DemiBold
                color: Theme.textMain
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Repeater {
                model: ["Beats Signature", "Bass Boost", "Vocal Clarity", "Flat"]

                delegate: Rectangle {
                    id: btn
                    Layout.fillWidth: true
                    implicitHeight: 24
                    radius: Theme.radiusSm
                    color: root.currentProfile === modelData ? Theme.accent : (btnMouse.containsMouse ? Theme.bgCardHover : Theme.bgDark)

                    Text {
                        anchors.centerIn: parent
                        text: {
                            if (modelData === "Beats Signature") return "Imza";
                            if (modelData === "Bass Boost") return "Bas+";
                            if (modelData === "Vocal Clarity") return "Vokal";
                            return "Flat";
                        }
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        font.weight: root.currentProfile === modelData ? Font.Bold : Font.Normal
                        color: root.currentProfile === modelData ? "#ffffff" : Theme.textMuted
                    }

                    MouseArea {
                        id: btnMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.profileSelected(modelData)
                    }
                }
            }
        }
    }
}
