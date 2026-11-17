import QtQuick
QtObject {
    property Instantiator w: Instantiator {
        model: 2
        onObjectAdded: (i, o) => console.log("added " + i)
        delegate: Window { visible: true; width: 100; height: 100; Component.onCompleted: console.log("win") }
    }
    Component.onCompleted: console.log("root done, count " + w.count)
}
