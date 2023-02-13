import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    objectName: "fatalErrorPage"

    property string errorMessage

    //: fatal error page title
    //% "Error"
    mainTitle: qsTrId("whisperfish-fatal-error-title")
    mainDescription: errorMessage

    //: generic hint on what to do after a fatal error occurred
    //: (error message will be shown separately)
    //% "Please restart Whisperfish. If the problem persists and appears "
    //% "to be an issue with Whisperfish, please report the issue."
    detailedDescription: qsTrId("whisperfish-fatal-error-hint")
    iconSource: "image://theme/icon-l-attention"
    pageTitle: ""

    Component.onCompleted: {
        console.log("[FATAL] error occurred: "+errorMessage)
    }
}
