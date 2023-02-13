import QtQuick 2.5
import Sailfish.Silica 1.0
import Nemo.DBus 2.0
import "../components"
import "../js/countries.js" as CallingCodes

BlockingInfoPageBase {
    id: root
    objectName: "registerSecondaryPage"

    pageTitle: "" // xx("Step 2.1")

    //: register as secondary device qr page title
    //% "Link as secondary device"
    mainTitle: qsTrId("whisperfish-registration-secondary-title")

    //: User instructions
    //% "Please scan the QR code below using the Signal app."
    mainDescription: qsTrId("whisperfish-register-linked-message")

    Connections {
        target: SetupWorker
        onRegistrationSuccess: {
            // TODO actually send this signal from the backend
            console.log("WARNING handling SetupWorker.registrationSuccess is not implemented yet")
            // showMainPage(PageStackAction.Animated)
        }
        onSetupComplete: {
            if (SetupWorker.registered) {
                showMainPage()
            } else {
                // this should never be reached when not registered
                //: fatal error when trying to unlock the db when not registered
                //% "You are not registered."
                showFatalError(qsTrId("whisperfish-fatal-error-msg-not-registered"))
            }
        }
    }

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        Image {
            anchors {
                left: parent.left
                right: parent.right
            }

            height: Math.min(Screen.width, Screen.height) - 4*Theme.horizontalPageMargin
            cache: false
            fillMode: Image.PreserveAspectFit
            source: Prompt.linkingQR

            BusyIndicator {
                anchors.centerIn: parent
                size: BusyIndicatorSize.Large
                running: parent.status != Image.Ready
            }
        }
    }
}
