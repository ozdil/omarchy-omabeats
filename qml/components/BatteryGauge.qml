import QtQuick
import QtQuick.Layouts
import "../theme"

Rectangle {
    id: root

    property string label: "Sol"
    property string iconText: Theme.iconEarbudLeft
    property int batteryLevel: 85
    property bool isCharging: false
    property bool isPresent: batteryLevel >= 0

    implicitWidth: 100
    implicitHeight: 74
    radius: Theme.radiusMd
    color: Theme.bgCard
    border.color: isCharging ? Theme.accentGreen : Theme.border
    border.width: isCharging ? 1.5 : 1

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 4

        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            Text {
                text: root.iconText
                font.family: Theme.iconFont
                font.pixelSize: 14
                color: root.isPresent ? Theme.textMain : Theme.textDim
            }

            Text {
                text: root.label
                font.family: Theme.fontFamily
                font.pixelSize: 11
                font.weight: Font.DemiBold
                color: Theme.textMuted
                Layout.fillWidth: true
            }

            Text {
                visible: root.isCharging
                text: Theme.iconBatteryBolt
                font.family: Theme.iconFont
                font.pixelSize: 12
                color: Theme.accentGreen

                SequentialAnimation on opacity {
                    running: root.isCharging
                    loops: Animation.Infinite
                    PropertyAnimation { to: 0.3; duration: 800; easing.type: Easing.InOutQuad }
                    PropertyAnimation { to: 1.0; duration: 800; easing.type: Easing.InOutQuad }
                }
            }
        }

        Text {
            text: root.isPresent ? (root.batteryLevel + "%") : "--"
            font.family: Theme.fontFamily
            font.pixelSize: 16
            font.weight: Font.Bold
            color: {
                if (!root.isPresent) return Theme.textDim;
                if (root.batteryLevel > 40) return Theme.accentGreen;
                if (root.batteryLevel > 15) return Theme.accentYellow;
                return Theme.accent;
            }
        }

        // Mini battery bar
        Rectangle {
            Layout.fillWidth: true
            height: 4
            radius: 2
            color: Theme.bgDark

            Rectangle {
                width: root.isPresent ? (parent.width * (root.batteryLevel / 100.0)) : 0
                height: parent.height
                radius: 2
                color: {
                    if (root.batteryLevel > 40) return Theme.accentGreen;
                    if (root.batteryLevel > 15) return Theme.accentYellow;
                    return Theme.accent;
                }
            }
        }
    }
}
