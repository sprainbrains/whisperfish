import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    //: unlock page title
    //% "Unlock"
    pageTitle: qsTrId("whisperfish-unlock-page-title")
    //: unlock page welcome title, centered on screen
    //% "Whisperfish"
    mainTitle: qsTrId("whisperfish-unlock-welcome-title")
    //: unlock page password prompt
    //% "Please enter your password to unlock your conversations."
    mainDescription: qsTrId("whisperfish-unlock-password-prompt")

    property bool _inputIsValid: !passwordField.errorHighlight &&
                                 SetupWorker.registered

    signal accept
    onAccept: {
        if (!SetupWorker.registered) {
            // this page should never be reached when not registered
            //: fatal error when trying to unlock the db when not registered
            //% "You are not registered."
            showFatalError(qsTrId("whisperfish-fatal-error-msg-not-registered"))
            return
        } else if (!_inputIsValid) {
            return
        }

        busy = true
        Prompt.password(passwordField.text)
        // TODO Until we have a way of knowing if the entered
        // password was correct, we use the timer to continue
        // to the main page if no password prompt interrupts it.
        waitThenUnlock.restart()
    }

    Connections {
        // Receives a new password prompt if the password was incorrect.
        id: validationConnection
        target: Prompt
        onPromptPassword: {
            busy = false
            waitThenUnlock.stop()
            passwordField.text = ''
            // TODO give haptic feedback

            //: input field placeholder after failed attempt to unlock (keep it short)
            //% "Please try again"
            passwordField.placeholderText = qsTrId("whisperfish-unlock-try-again")
        }
    }

    Timer {  // TO BE REMOVED
        id: waitThenUnlock
        // If nothing happens in this time, we assume the
        // password was correct. N (3) invalid attempts result
        // in a fatal error, handled in mainWindow.
        interval: 1000
        running: false
        onTriggered: {
            mainWindow.showMainPage(PageStackAction.Animated)
        }
    }

    Column {
        width: parent.width
        spacing: 1.5*Theme.paddingLarge

        PasswordField {
            id: passwordField
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width - 2*Theme.horizontalPageMargin
            inputMethodHints: Qt.ImhNoPredictiveText
            validator: RegExpValidator{ regExp: /.{6,}/ }
            //: Password label
            //% "Password"
            label: qsTrId("whisperfish-password-label")
            //: password placeholder
            //% "Your password"
            placeholderText: qsTrId("whisperfish-password-placeholder")
            placeholderColor: Theme.highlightColor
            color: _inputIsValid ? Theme.primaryColor : Theme.highlightColor
            focus: true
            EnterKey.iconSource: "image://theme/icon-m-enter-accept"
            EnterKey.onClicked: accept()
        }

        Button {
            //: unlock button label
            //% "Unlock"
            text: qsTrId("whisperfish-unlock-button-label")
            enabled: _inputIsValid && !busy
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
