import QtQuick
import Quickshell
import Quickshell.Io
import "theme"

ShellRoot {
    id: shellRoot

    FloatingWindow {
        id: win
        title: "OmaBeats - Beats Control Hub"
        implicitWidth: 390
        implicitHeight: 650
        visible: true
        color: Theme.bgBase

        Dashboard {
            anchors.fill: parent
        }
    }

    IpcHandler {
        target: "ozdil.omabeats"

        function toggle(): bool {
            win.visible = !win.visible;
            return win.visible;
        }

        function show(): bool {
            win.visible = true;
            return true;
        }

        function hide(): bool {
            win.visible = false;
            return false;
        }
    }
}
