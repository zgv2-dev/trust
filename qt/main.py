# Qt Quick variant: BENCH_WINDOWS widgets, one transparent window each, or all in one click-through
# overlay window when BENCH_MODE=overlay. QML animates every frame; Python only loads it.
import os
import sys
from pathlib import Path

from PySide6.QtCore import QRect
from PySide6.QtGui import QGuiApplication, QRegion
from PySide6.QtQml import QQmlApplicationEngine

app = QGuiApplication(sys.argv)
engine = QQmlApplicationEngine()
engine.setInitialProperties({
    "count": int(os.environ.get("BENCH_WINDOWS", "4")),
    "overlay": os.environ.get("BENCH_MODE") == "overlay",
    # BENCH_FREEZE: draw that instant forever, for screenshots; -1 = animate
    "freeze": float(os.environ.get("BENCH_FREEZE", "-1")),
})
engine.load(Path(__file__).with_name("main.qml"))
if not engine.rootObjects():
    sys.exit(1)
if os.environ.get("BENCH_MODE") == "overlay":
    # input mask = the panels: clicks elsewhere fall through to any app below, panels stay clickable
    mask = QRegion()
    for i in range(int(os.environ.get("BENCH_WINDOWS", "4"))):
        mask += QRegion(QRect(40 + 260 * (i % 4), 40 + 140 * (i // 4), 240, 110))
    for window in app.topLevelWindows():
        window.setMask(mask)
# the bench reads "fps N" lines and one "first-frame" line from stdout
engine.rootObjects()[0].fps.connect(lambda n: print(f"fps {n}", flush=True))
first = app.topLevelWindows()[0]
def on_first_frame():
    print("first-frame", flush=True)
    first.frameSwapped.disconnect(on_first_frame)
first.frameSwapped.connect(on_first_frame)
sys.exit(app.exec())
