import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    objectName: "unlockPage"

    property bool autologinDone: false

    onStatusChanged: {
        if(status === PageStatus.Active) {
            if(!autologinDone && SettingsBridge.plaintext_password) {
                autologinDone = true
                passwordField.text = SettingsBridge.plaintext_password
                accept()
            }
        }
    }

    //: unlock page title
    //% "Unlock"
    pageTitle: qsTrId("whisperfish-unlock-page-title")
    //: unlock page welcome title, centered on screen
    //% "Whisperfish"
    mainTitle: qsTrId("whisperfish-unlock-welcome-title")
    //: unlock page password prompt
    //% "Please enter your password to unlock your conversations."
    mainDescription: qsTrId("whisperfish-unlock-password-prompt")

    property bool _canAccept: !passwordField.errorHighlight &&
                              passwordField.text.length > 0 &&
                              SetupWorker.registered

    signal accept
    onAccept: {
        if (!SetupWorker.registered) {
            // this page should never be reached when not registered
            //: fatal error when trying to unlock the db when not registered
            //% "You are not registered."
            showFatalError(qsTrId("whisperfish-fatal-error-msg-not-registered"))
            return
        } else if (!_canAccept) {
            return
        }

        busy = true
        Prompt.password(passwordField.text)

        // We expect a SetupWorker.setupComplete signal
        // when the database is ready. N (3) invalid attempts result
        // in a fatal error, handled in mainWindow.
    }

    Connections {
        // Receives a new password prompt if the password was incorrect.
        id: validationConnection
        target: Prompt
        onPromptPassword: {
            busy = false
            passwordField.text = ''
            // TODO give haptic feedback

            //: input field placeholder after failed attempt to unlock (keep it short)
            //% "Please try again"
            passwordField.placeholderText = qsTrId("whisperfish-unlock-try-again")
        }
    }

    Connections {
        target: SetupWorker
        onSetupComplete: mainWindow.showMainPage(PageStackAction.Animated)
    }

    Column {
        width: parent.width
        spacing: 1.5*Theme.paddingLarge

        PasswordField {
            id: passwordField
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width - 2*Theme.horizontalPageMargin
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhSensitiveData
            validator: RegExpValidator{ regExp: /|.{6,}/ }
            //: Password label
            //% "Password"
            label: qsTrId("whisperfish-password-label")
            //: password placeholder
            //% "Your password"
            placeholderText: qsTrId("whisperfish-password-placeholder")
            focus: true
            EnterKey.iconSource: "image://theme/icon-m-enter-accept"
            EnterKey.onClicked: accept()
        }

        Button {
            //: unlock button label
            //% "Unlock"
            text: qsTrId("whisperfish-unlock-button-label")
            enabled: _canAccept && !busy
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
