import sys
from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine
from PySide6.QtCore import QTimer
app = QGuiApplication(sys.argv); e = QQmlApplicationEngine(); e.load(sys.argv[1]); print("roots", e.rootObjects(), flush=True)
QTimer.singleShot(1500, app.quit); app.exec()
