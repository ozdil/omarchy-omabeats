pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: root

    property string themeName: "default"
    property bool isDarkTheme: true

    // Dynamic Omarchy color palette
    property color bgDark: "#13141c"
    property color bgBase: "#16161e"
    property color bgSurface: "#1a1b26"
    property color bgCard: "#202330"
    property color bgCardHover: "#282c3f"
    property color bgCardActive: "#32374d"
    property color border: "#2f354a"
    property color borderLight: "#414868"

    property color textMain: "#c0caf5"
    property color textMuted: "#7982a9"
    property color textDim: "#545c7e"

    // Accent colors
    property color accent: "#e11d48" // Beats red accent
    property color accentHover: "#f43f5e"
    property color accentBlue: "#7aa2f7"
    property color accentGreen: "#9ece6a"
    property color accentYellow: "#e0af68"
    property color accentOrange: "#ff9e64"
    property color accentPurple: "#bb9af7"

    // Radii
    readonly property int radiusSm: 6
    readonly property int radiusMd: 10
    readonly property int radiusLg: 14

    // Strict Typography as per Omarchy Standards
    readonly property string fontFamily: "JetBrainsMono Nerd Font, JetBrains Mono, monospace"
    readonly property string monoFont: "JetBrainsMono Nerd Font, JetBrains Mono, monospace"
    readonly property string iconFont: "JetBrainsMono Nerd Font, monospace"

    // Themeable Nerd Font Glyph Constants (Zero Unicode Emojis)
    readonly property string iconHeadphones: "󰋋"
    readonly property string iconEarbudLeft: "󰥈"
    readonly property string iconEarbudRight: "󰥉"
    readonly property string iconBatteryCase: "󰂑"
    readonly property string iconBatteryBolt: "󱐋"
    readonly property string iconBluetoothConnected: "󰂯"
    readonly property string iconBluetoothDisconnected: "󰂲"
    readonly property string iconAncNoise: "󱡥"
    readonly property string iconAncTransparency: "󰂚"
    readonly property string iconAncOff: "󰂛"
    readonly property string iconAncAdaptive: "󰥒"
    readonly property string iconMic: "󰍬"
    readonly property string iconMicMute: "󰍭"
    readonly property string iconChime: "󰂞"
    readonly property string iconSettings: "󰒲"
    readonly property string iconRefresh: "󰑐"
    readonly property string iconSliders: "󰦨"
    readonly property string iconCheck: "󰄬"
    readonly property string iconMusic: "󰝚"
    readonly property string iconWaveform: "󰠃"
    readonly property string iconSignalHigh: "󰤨"
    readonly property string iconTestMode: "󰙨"

    // Filesystem Paths for Omarchy System Theme
    readonly property string homeDir: Quickshell.env("HOME")
    readonly property string omarchyStateDir: homeDir + "/.local/state/omarchy/current"

    property string lastLoadedRaw: ""

    function loadColors(raw) {
        if (!raw || raw.trim().length === 0 || raw === lastLoadedRaw) return;
        lastLoadedRaw = raw;

        var dict = {};
        var lines = String(raw).split("\n");
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i].trim();
            if (!line || line.charAt(0) === '#') continue;
            var match = line.match(/^([A-Za-z0-9_-]+)\s*=\s*["']?([^"'\r\n]+?)["']?\s*(?:#.*)?$/);
            if (match) {
                dict[match[1].toLowerCase()] = match[2].trim();
            }
        }

        var mode = dict["mode"] || "dark";
        root.isDarkTheme = (mode !== "light");

        var base = dict["background"] || dict["bg"] || (root.isDarkTheme ? "#16161e" : "#f0f2f5");
        var fg = dict["foreground"] || dict["fg"] || (root.isDarkTheme ? "#c0caf5" : "#1a1b26");
        var acc = dict["accent"] || dict["primary"] || "#e11d48";

        root.bgBase = Qt.color(base);
        root.textMain = Qt.color(fg);
        root.accent = Qt.color(acc);

        if (root.isDarkTheme) {
            root.bgDark = Qt.darker(root.bgBase, 1.25);
            root.bgSurface = Qt.lighter(root.bgBase, 1.15);
            root.bgCard = Qt.lighter(root.bgBase, 1.30);
            root.bgCardHover = Qt.lighter(root.bgBase, 1.45);
            root.border = Qt.lighter(root.bgBase, 1.6);
            root.textMuted = Qt.rgba(root.textMain.r, root.textMain.g, root.textMain.b, 0.65);
            root.textDim = Qt.rgba(root.textMain.r, root.textMain.g, root.textMain.b, 0.40);
        } else {
            root.bgDark = Qt.lighter(root.bgBase, 1.05);
            root.bgSurface = Qt.darker(root.bgBase, 1.05);
            root.bgCard = "#ffffff";
            root.bgCardHover = Qt.darker(root.bgBase, 1.08);
            root.border = Qt.darker(root.bgBase, 1.2);
            root.textMuted = Qt.rgba(root.textMain.r, root.textMain.g, root.textMain.b, 0.65);
            root.textDim = Qt.rgba(root.textMain.r, root.textMain.g, root.textMain.b, 0.40);
        }
    }

    Timer {
        interval: 10000
        running: true
        repeat: true
        onTriggered: {
            if (!themeFileReader.running) {
                themeFileReader.running = true;
            }
        }
    }

    Process {
        id: themeFileReader
        command: ["/usr/bin/cat", root.omarchyStateDir + "/colors.conf"]
        running: true
        stdout: StdioCollector {
            onDataChanged: {
                root.loadColors(text);
            }
        }
    }
}
