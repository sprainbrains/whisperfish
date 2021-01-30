import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: root
    property string errorMessage

    // block any navigation
    backNavigation: false
    forwardNavigation: false
    showNavigationIndicator: false

    SilicaListView {
        anchors.fill: parent
        ViewPlaceholder {
            visible: true
            enabled: true
            text: qsTr("Error")
            hintText: errorMessage + "\n\n" + qsTr("Please restart Whisperfish.")
        }
    }

    Component.onCompleted: {
        console.log("[FATAL] error occurred: "+errorMessage)
    }
}
