import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"

// NOTE The registration process should actually be a chain
// of dialogs, to be perfectly Sailfish-y. This is not possible,
// though, because we have to wait for a signal that allows
// us to continue.

BlockingInfoPageBase {
    id: root
    pageTitle: "" // qsTr("Step 1")
    mainTitle: qsTr("Welcome to Whisperfish")
    mainDescription: qsTr("Set a new password to secure your conversations.")

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
        onPromptPhoneNumber: pageStack.push(Qt.resolvedUrl("RegisterPage.qml"))
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
            placeholderText: qsTr("Your new password")
            placeholderColor: Theme.highlightColor
            color: errorHighlight ? Theme.highlightColor : Theme.primaryColor
            focus: true
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
                       qsTr("Repeated password") :
                       qsTr("Passwords do not match")
            placeholderText: qsTr("Repeat your new password")
            placeholderColor: Theme.highlightColor
            color: _inputIsValid ? Theme.primaryColor : Theme.highlightColor
            EnterKey.iconSource: _inputIsValid ?
                                     "image://theme/icon-m-enter-accept" :
                                     "image://theme/icon-m-enter-close"
            EnterKey.onClicked: accept()
        }

        Button {
            text: qsTr("Continue")
            enabled: _inputIsValid
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
