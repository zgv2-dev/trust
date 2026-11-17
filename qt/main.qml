// Same widget as the other variants, in Qt Quick. Glow comes from MultiEffect shadows (GPU).
import QtQuick
import QtQuick.Shapes
import QtQuick.Effects

QtObject {
    id: root
    required property int count
    required property bool overlay
    required property real freeze
    readonly property color accent: "#50ff78"
    property FontLoader mono: FontLoader { source: "../fonts/SpaceMono-Regular.ttf" }
    signal fps(int n)

    component Panel: Item {
        id: panel
        width: 240; height: 110
        property real clock: 0
        readonly property real t: root.freeze >= 0 ? root.freeze : clock
        FrameAnimation { running: true; onTriggered: panel.clock += frameTime }
        Rectangle {
            anchors.fill: parent; radius: 4
            color: Qt.rgba(3 / 255, 10 / 255, 3 / 255, 0.25)
        }
        Rectangle { // the border, with a tiny outer glow (blurred shadow of the border itself)
            anchors.fill: parent; radius: 4
            color: "transparent"
            border.color: Qt.rgba(0.31, 1, 0.47, 0.35)
            layer.enabled: true
            layer.effect: MultiEffect { shadowEnabled: true; shadowColor: Qt.rgba(0.31, 1, 0.47, 0.45); shadowBlur: 0.3; shadowHorizontalOffset: 0; shadowVerticalOffset: 0; autoPaddingEnabled: true }
        }
        Item {
            anchors.fill: parent; anchors.margins: 10
            layer.enabled: true
            layer.effect: MultiEffect { shadowEnabled: true; shadowColor: root.accent; shadowBlur: 0.4; shadowHorizontalOffset: 0; shadowVerticalOffset: 0 }
            Text { text: "MOTORS"; color: root.accent; font.family: root.mono.font.family }
            Row {
                y: 18; spacing: 8
                Repeater {
                    model: 4
                    Column {
                        required property int index
                        property real f: 0.5 + 0.45 * Math.sin(panel.t * 1.3 + index) // fake PWM
                        Shape {
                            width: 44; height: 44
                            ShapePath {
                                strokeColor: Qt.rgba(0.31, 1, 0.47, 0.5); fillColor: "transparent"
                                PathAngleArc { centerX: 22; centerY: 22; radiusX: 20; radiusY: 20; startAngle: 0; sweepAngle: 360 }
                            }
                            ShapePath {
                                strokeColor: "transparent"; fillColor: Qt.rgba(0.31, 1, 0.47, 0.6)
                                startX: 22; startY: 22
                                PathAngleArc { centerX: 22; centerY: 22; radiusX: 20; radiusY: 20; startAngle: -90; sweepAngle: f * 360; moveToStart: false }
                            }
                        }
                        Text {
                            width: 44; horizontalAlignment: Text.AlignHCenter
                            text: Math.round(1000 + f * 1000); color: root.accent; font.family: root.mono.font.family
                        }
                    }
                }
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
            x: root.overlay ? 0 : 40 + 260 * (index % 4)
            y: root.overlay ? 0 : 40 + 140 * Math.floor(index / 4)
            width: root.overlay ? Screen.width : 240
            height: root.overlay ? Screen.height : 110
            onFrameSwapped: frames++
            Timer { interval: 1000; repeat: true; running: true; onTriggered: { root.fps(win.frames); win.frames = 0 } }
            Repeater {
                model: root.overlay ? root.count : 1
                Panel {
                    required property int index
                    x: root.overlay ? 40 + 260 * (index % 4) : 0
                    y: root.overlay ? 40 + 140 * Math.floor(index / 4) : 0
                }
            }
        }
    }
}
