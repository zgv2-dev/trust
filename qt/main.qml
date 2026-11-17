// Same widget as the other variants, in Qt Quick: the reference drawing code (widget.js) on two
// Canvas items per panel, static parts painted once and wedges/numbers every frame.
import QtQuick
import "widget.js" as Look

QtObject {
    id: root
    required property int count
    required property bool overlay
    required property real freeze
    readonly property int margin: 8 // room around the panel for its outer glow
    property FontLoader mono: FontLoader { source: "../fonts/SpaceMono-Regular.ttf" }
    signal fps(int n)

    component Panel: Item {
        id: panel
        width: Look.W; height: Look.H
        property real clock: 0
        readonly property real t: root.freeze >= 0 ? root.freeze : clock
        FrameAnimation { running: true; onTriggered: { panel.clock += frameTime; moving.requestPaint() } }
        Canvas {
            x: -root.margin; y: -root.margin; width: Look.W + 2 * root.margin; height: Look.H + 2 * root.margin
            // repaint once the font is in, so the title isn't left in a fallback font
            property bool fontReady: root.mono.status === FontLoader.Ready
            onFontReadyChanged: requestPaint()
            onPaint: {
                const g = getContext("2d");
                g.reset();
                g.translate(root.margin, root.margin);
                Look.drawStatic(g);
            }
        }
        Canvas {
            id: moving
            x: -root.margin; y: -root.margin; width: Look.W + 2 * root.margin; height: Look.H + 2 * root.margin
            onPaint: {
                const g = getContext("2d");
                g.reset();
                g.translate(root.margin, root.margin);
                Look.drawDynamic(g, panel.t);
            }
        }
    }

    property Instantiator windows: Instantiator {
        model: root.overlay ? 1 : root.count
        delegate: Window {
            id: win
            required property int index
            property int frames: 0
            visible: true
            color: "transparent"
            // no Qt.Tool on macOS: tool windows there are panels that hide while the app isn't active
            flags: Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | (Qt.platform.os === "osx" ? 0 : Qt.Tool)
            x: root.overlay ? 0 : 40 + 260 * (index % 4) - root.margin
            y: root.overlay ? 0 : 40 + 140 * Math.floor(index / 4) - root.margin
            width: root.overlay ? Screen.width : Look.W + 2 * root.margin
            height: root.overlay ? Screen.height : Look.H + 2 * root.margin
            onFrameSwapped: frames++
            Timer { interval: 1000; repeat: true; running: true; onTriggered: { root.fps(win.frames); win.frames = 0 } }
            Repeater {
                model: root.overlay ? root.count : 1
                Panel {
                    required property int index
                    x: root.overlay ? 40 + 260 * (index % 4) : root.margin
                    y: root.overlay ? 40 + 140 * Math.floor(index / 4) : root.margin
                }
            }
        }
    }
}
