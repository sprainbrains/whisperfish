import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: root
    objectName: "landingPage"

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
            pageStack.push(Qt.resolvedUrl("RegisterPage.qml"))
        } else if (action === "verify") {
            pageStack.push(Qt.resolvedUrl("VerifyRegistrationPage.qml"))
        } else if (action === "unlock") {
            pageStack.push(Qt.resolvedUrl("UnlockPage.qml"))
        } else if (action === "prepareRegistration") {
            pageStack.push(Qt.resolvedUrl("SetupPasswordPage.qml"))
        } else if (action === "showMain") {
            showMainPage(PageStackAction.Animated)
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
        // Registration and verification are handled in the respective
        // pages. We still catch these signals to allow continuing
        // an interrupted registration process.
        onPromptPhoneNumber: nextAction = "register"
        onPromptVerificationCode: nextAction = "verify"
        onPromptPassword: {
            if (SetupWorker.registered) {
                nextAction = "unlock"
            } else {
                nextAction = "prepareRegistration"
            }
        }
    }

    Connections {
        target: SetupWorker
        onSetupComplete: nextAction = "showMain"
    }

    RemorsePopup { id: setupRemorse }

    BusyLabel {
        id: waitingPlaceholder

        //: welcome text shown when startup takes a long time
        //% "Welcome"
        text: qsTrId("whisperfish-startup-placeholder-title")
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
