import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

// NOTE The registration process should actually be a chain
// of dialogs, to be perfectly Sailfish-y. This is not possible,
// though, because we have to wait for a signal that allows
// us to continue.

BlockingInfoPageBase {
    id: root
    pageTitle: "" // xx("Step 1")

    //: welcome screen title when creating a new database
    //% "Welcome to Whisperfish"
    mainTitle: qsTrId("whisperfish-initial-setup-welcome-title")

    //: new password setup prompt
    //% "Set a new password to secure your conversations."
    mainDescription: qsTrId("whisperfish-setup-password-prompt")

    //: Whisperfish password informational message
    //% "Whisperfish stores identity keys, session state, and local message data encrypted on disk. The password you set is not stored anywhere and you will not be able to restore your data if you lose your password. Note: Attachments are currently stored unencrypted. You can disable storing of attachments in the Settings page."
    detailedDescription: qsTrId("whisperfish-password-info")
    squashDetails: true

    property bool _inputIsValid: (!password1Field.errorHighlight &&
                                !password2Field.errorHighlight &&
                                password1Field.text === password2Field.text)

    signal accept
    onAccept: {
        if (!_inputIsValid) return
        Prompt.password(password1Field.text)
        busy = true // wait for the backend to prompt the next step
    }

    Connections {
        // We wait till the backend calls to continue.
        target: Prompt
        onPromptPhoneNumber: {
            root.forceActiveFocus() // to close the keyboard
            pageStack.replace(Qt.resolvedUrl("RegisterPage.qml"),
                              PageStackAction.Animated)
        }
    }

    Column {
        width: parent.width
        spacing: 1.5*Theme.paddingLarge

        PasswordField {
            id: password1Field
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width - 2*Theme.horizontalPageMargin
            inputMethodHints: Qt.ImhNoPredictiveText
            validator: RegExpValidator{ regExp: /.{6,}/ }
            //: Password label
            //% "Password"
            label: qsTrId("whisperfish-password-label")
            //: New password input placeholder
            //% "Your new password"
            placeholderText: qsTrId("whisperfish-new-password-placeholder")
            placeholderColor: Theme.highlightColor
            color: errorHighlight ? Theme.highlightColor : Theme.primaryColor
            EnterKey.iconSource: "image://theme/icon-m-enter-next"
            EnterKey.onClicked: password2Field.forceActiveFocus()
        }

        PasswordField {
            id: password2Field
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width - 2*Theme.horizontalPageMargin
            inputMethodHints: Qt.ImhNoPredictiveText
            validator: RegExpValidator{ regExp: /.{6,}/ }
            label: (text === '' || _inputIsValid) ?
                       //: repeated password input label
                       //% "Repeated password"
                       qsTrId("whisperfish-password-repeated-label") :
                       //: repeated password input label if passwords don't match
                       //% "Passwords do not match"
                       qsTrId("whisperfish-password-repeated-label-wrong")
            //: Repeated new password input placeholder
            //% "Repeat your new password"
            placeholderText: qsTrId("whisperfish-new-password-repeat-placeholder")
            placeholderColor: Theme.highlightColor
            color: _inputIsValid ? Theme.primaryColor : Theme.highlightColor
            EnterKey.iconSource: _inputIsValid ?
                                     "image://theme/icon-m-enter-accept" :
                                     "image://theme/icon-m-enter-close"
            EnterKey.onClicked: accept()
        }

        Button {
            //: continue button label
            //% "Continue"
            text: qsTrId("whisperfish-continue-button-label")
            enabled: _inputIsValid && !busy
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
