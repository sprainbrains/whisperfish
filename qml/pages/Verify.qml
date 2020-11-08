import QtQuick 2.2
import Sailfish.Silica 1.0

Dialog {
    id: verifyDialog
    objectName: "verifyDialog"
    property string code

    canAccept: !codeField.errorHighlight

    onDone: {
        if (result == DialogResult.Accepted && !codeField.errorHighlight) {
            code = codeField.text
            Prompt.verificationCode(code)
        }
    }

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        DialogHeader {
            //: Verify code accept
            //% "Verify"
            acceptText: qsTrId("whisperfish-verify-code-accept")
        }

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            font.bold: true
            //: Verify code page title
            //% "Verify Device"
            text: qsTrId("whisperfish-verify-code-title")
        }

        TextField {
            id: codeField
            width: parent.width
            inputMethodHints: Qt.ImhDigitsOnly | Qt.ImhNoPredictiveText
            validator: RegExpValidator{ regExp: /[0-9]+/;}
            //: Verify code label
            //% "Code"
            label: qsTrId("whisperfish-verify-code-label")
            //: Verify code placeholder
            //% "123456"
            placeholderText: qsTrId("whisperfish-verify-code-placeholder")
            placeholderColor: Theme.highlightColor
            horizontalAlignment: TextInput.AlignLeft
            color: errorHighlight? "red" : Theme.primaryColor
            EnterKey.onClicked: parent.focus = true
        }

        TextArea {
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width
            horizontalAlignment: TextEdit.Center
            readOnly: true
            visible:  SetupWorker.useVoice
            //: Voice verification code instructions
            //% "Signal will call you with a 6-digit verification code. Please enter it here."
            text: qsTrId("whisperfish-voice-verify-code-instructions")
        }

        TextArea {
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width
            horizontalAlignment: TextEdit.Center
            readOnly: true
            visible:  ! SetupWorker.useVoice
            //: Text verification code instructions
            //% "Signal will text you a 6-digit verification code. Please enter it here, using only numbers."
            text: qsTrId("whisperfish-text-verify-code-instructions")
        }

    }
}
