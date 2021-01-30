import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    property string errorMessage

    pageTitle: ""
    mainTitle: qsTr("Error")
    mainDescription: errorMessage
    detailedDescription: qsTr("Please restart Whisperfish. If the problem persists and appears "+
                              "to be an issue with Whisperfish, please report the issue.")
    iconSource: "image://theme/icon-l-attention"

    Component.onCompleted: {
        console.log("[FATAL] error occurred: "+errorMessage)
    }
}
