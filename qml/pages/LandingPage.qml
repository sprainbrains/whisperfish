import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: root
    property bool readyToGo: false
    property string nextAction: "none"

    function handleNextStep() {
        if (!readyToGo || nextAction == "none") {
            return
        }

        var action = nextAction
        readyToGo = false
        nextAction = "none"
        if (action === "register") {
            pageStack.push(Qt.resolvedUrl("Register.qml"))
        } else if (action === "verify") {
            pageStack.push(Qt.resolvedUrl("Verify.qml"))
        } else if (action === "unlock") {
            pageStack.push(Qt.resolvedUrl("UnlockPage.qml"))
        }
    }

    onNextActionChanged: handleNextStep()
    onStatusChanged: {
        if (status === PageStatus.Active) {
            pageStack.completeAnimation() // abort any running animation

            // we have to wait until this page is ready because
            // we can't push another page on the stack while the current
            // page is being built
            readyToGo = true
            handleNextStep()
        } else {
            readyToGo = false
        }
    }

    Connections {
        target: Prompt
        onPromptPhoneNumber: nextAction = "register"
        onPromptVerificationCode: nextAction = "verify"
        onPromptPassword: nextAction = "unlock"
    }

    Connections {
        // FIXME Registration is not yet tested! This code
        // is moved from Main.qml.
        target: SetupWorker
        onRegistrationSuccess: {
            //: Registration complete remorse message
            //% "Registration complete!"
            setupRemorse.execute(qsTrId("whisperfish-registration-complete"), function() { console.log("Registration complete") })
        }
        onInvalidDatastore: {
            //: Failed to setup datastore error message
            //% "ERROR - Failed to setup datastore"
            setupRemorse.execute(qsTrId("whisperfish-error-invalid-datastore"), function() { console.log("Failed to setup datastore") })
        }
        onInvalidPhoneNumber: {
            //: Invalid phone number error message
            //% "ERROR - Invalid phone number registered with Signal"
            setupRemorse.execute(qsTrId("whisperfish-error-invalid-number"), function() { console.log("Invalid phone number registered with Signal") })
        }
    }

    RemorsePopup { id: setupRemorse }

    BusyLabel {
        id: waitingPlaceholder
        text: qsTr("Welcome")
        running: false
        opacity: running ? 1.0 : 0.0
        Behavior on opacity { FadeAnimator { } }
    }

    Timer {
        // Delay showing "Welcome". We should
        // already be on the next page when this is triggered -
        // but if not, we'll let the user see something.
        running: true
        interval: 500
        onTriggered: waitingPlaceholder.running = true
    }
}
