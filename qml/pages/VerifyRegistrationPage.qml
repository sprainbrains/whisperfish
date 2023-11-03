import QtQuick 2.5
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    objectName: "verifyRegistrationPage"

    pageTitle: "" // xx("Step 3")
    //: verify registration page title
    //% "Verify"
    mainTitle: qsTrId("whisperfish-verify-page-title")
    //: verify registration prompt
    //% "Please enter the code you received from Signal."
    mainDescription: qsTrId("whisperfish-verify-code-prompt")

    detailedDescription: SetupWorker.useVoice ?
                             //: verify registration instructions: voice
                             //% "Signal should have called you with a 6-digit "
                             //% "verification code. Please wait a moment, or "
                             //% "restart the process if you have not received a call."
                             qsTrId("whisperfish-verify-instructions-voice") :
                             //: verify registration instructions: text message
                             //% "Signal should have sent you a 6-digit verification "
                             //% "code via text message. Please wait a moment, or "
                             //% "restart the process if you have not received a message."
                             qsTrId("whisperfish-verify-instructions-sms")
    squashDetails: true

    property bool _canAccept: codeField.acceptableInput &&
                              codeField.text.length !== 0

    signal accept
    onAccept: {
        if (!_canAccept) return
        Prompt.verificationCode(codeField.text.replace('-', ''))
        busy = true // wait for the backend to prompt the next step
    }

    signal _retry
    on_Retry: {
        // TODO give haptic feedback
        //: verification: prompt to retry with a new code
        //% "Please retry with a valid code."
        mainDescription = qsTrId("whisperfish-verify-retry-prompt")
    }

    Connections {
        target: Prompt
        onPromptVerificationCode: _retry()
        // TODO handle a failure signal from the backend to
        // abort gracefully, i.e. ask the user to restart Whisperfish
    }

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
        spacing: 2*Theme.paddingLarge

        TextField {
            id: codeField
            width: parent.width - 4*Theme.horizontalPageMargin
            anchors.horizontalCenter: parent.horizontalCenter
            inputMethodHints: Qt.ImhDigitsOnly | Qt.ImhNoPredictiveText | Qt.ImhSensitiveData
            validator: RegExpValidator{ regExp: /|[0-9]{6}/; }
            //: verification code input label
            //% "Verification code"
            label: qsTrId("whisperfish-verify-code-input-label")
            //: verification code input placeholder
            //% "Code"
            placeholderText: qsTrId("whisperfish-verify-code-input-placeholder")
            horizontalAlignment: TextInput.AlignHCenter
            font.pixelSize: Theme.fontSizeLarge
            EnterKey.onClicked: {
                if (_canAccept) {
                    accept()
                }
            }
            errorHighlight: !_canAccept

            // For SFOS 3.4 compatibility
            Component.onCompleted: {
                if(codeField.rightItem !== undefined) {
                    _codeFieldLoader.active = true
                    codeField.rightItem = _codeFieldLoader.item
                    codeField.errorHighlight = false
                }
            }

            Loader {
                id: _codeFieldLoader
                active: false
                sourceComponent: Image {
                    width: codeField.font.pixelSize
                    height: codeField.font.pixelSize
                    source: "image://theme/icon-m-acknowledge?" + codeField.color
                    opacity: _canAccept ? 1.0 : 0.01
                    Behavior on opacity { FadeAnimation {} }
                }
            }
        }

        Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: Theme.paddingMedium
            width: childrenRect.width
            height: childrenRect.height

            Button {
                //: continue button label
                //% "Continue"
                text: qsTrId("whisperfish-continue-button-label")
                enabled: _canAccept
                onClicked: accept()
            }

            // TODO add second button to resend verification code
        }
    }
}
