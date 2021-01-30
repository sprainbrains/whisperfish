import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: root
    property bool readyToGo: false
    property string nextAction: "none"

    function handleNextStep() {
        if (!readyToGo || nextAction == "none") {
            return
        } else {
            readyToGo = false
        }

        if (nextAction == "register") {
            nextAction = "none" // we'll wait for 'verify'
            pageStack.push(Qt.resolvedUrl("Register.qml"))
        } else if (nextAction == "verify") {
            nextAction = "none" // we'll wait for 'unlock'
            pageStack.push(Qt.resolvedUrl("Verify.qml"))
        } else if (nextAction == "unlock") {
            nextAction = "verifyUnlocked" // we'll be back
            pageStack.push(Qt.resolvedUrl("UnlockPage.qml"))
        } else if (nextAction == "verifyUnlocked") {
            nextAction = "none"
            // TODO Until we have a way of knowing if the entered
            // password was correct, we use the timer to continue
            // to the main page if no password prompt interrupts it.
            waitThenUnlock.restart()
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
        onPromptPassword: {
            waitThenUnlock.stop()
            nextAction = "unlock"
        }
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
        onClientFailed: {
            //: Failed to setup signal client error message
            //% "ERROR - Failed to setup Signal client"
            setupRemorse.execute(qsTrId("whisperfish-error-setup-client"), function() { console.log("Failed to setup Signal client") })
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
        // already be on the next page when this is
        // triggered. We'll see it when we come back.
        running: true
        interval: 500
        onTriggered: waitingPlaceholder.running = true
    }

    Timer {  // TO BE REMOVED
        id: waitThenUnlock
        interval: 200 // If nothing happens in this time,
                      // we assume the password was correct.
                      // There is no notification after the third
                      // invalid attempt, though...
        running: false
        onTriggered: {
            mainWindow.showMainPage()
        }
    }
}
