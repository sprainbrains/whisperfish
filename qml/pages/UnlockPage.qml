import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    pageTitle: qsTr("Unlock")
    mainTitle: qsTr("Whisperfish")
    mainDescription: qsTr("Please enter your password to unlock your conversations.")

    property bool _inputIsValid: !passwordField.errorHighlight &&
                                 SetupWorker.registered

    signal accept
    onAccept: {
        if (!SetupWorker.registered) {
            // this page should never be reached when not registered
            showFatalError(qsTr("You are not registered."))
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
            passwordField.placeholderText = qsTr("Please try again")
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
            mainWindow.showMainPage(PageStackAction.Replace)
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
            placeholderText: qsTr("Your password")
            placeholderColor: Theme.highlightColor
            color: _inputIsValid ? Theme.primaryColor : Theme.highlightColor
            focus: true
            EnterKey.iconSource: "image://theme/icon-m-enter-accept"
            EnterKey.onClicked: accept()
        }

        Button {
            text: qsTr("Unlock")
            enabled: _inputIsValid && !busy
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
