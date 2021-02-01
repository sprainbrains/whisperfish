import QtQuick 2.5
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    pageTitle: "" // xx("Step 3")
    //: verify registration page title
    //% "Verify"
    mainTitle: qsTrId("whisperfish-verify-page-title")
    //: verify registration prompt
    //% "Please enter the code you received from Signal."
    mainDescription: qsTrId("whisperfish-verify-code-prompt")

    detailedDescription: SetupWorker.useVoice ?
                             //: verify registration instructions: voice
                             //% "Signal should have called you with a a 6-digit "
                             //% "verification code. Please wait a moment, or "
                             //% "restart the process if you have not received a call."
                             qsTrId("whisperfish-verify-instructions-voice") :
                             //: verify registration instructions: text message
                             //% "Signal should have sent you a 6-digit verification "
                             //% "code via text message. Please wait a moment, or "
                             //% "restart the process if you have not received a message."
                             qsTrId("whisperfish-verify-instructions-sms")
    squashDetails: true

    property bool _canAccept: !codeField.errorHighlight &&
                              codeField.text.length !== 0

    signal accept
    onAccept: {
        if (!_canAccept) return
        Prompt.verificationCode(codeField.text.replace('-', ''))
        busy = true // wait for the backend to prompt the next step

        // TODO We should receive a success/failure signal instead of
        // using this timer.
        waitThenUnlock.restart()
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
            console.log("registration complete")
            // TODO actually send this signal from the backend
            showMainPage(PageStackAction.Animated)
        }
        /* onSetupChanged: {
            // We assume the process has been successfully completed.
            if (SetupWorker.registered) {
                showMainPage(PageStackAction.Animated)
            }
        } */
    }

    Timer {  // TO BE REMOVED
        id: waitThenUnlock
        // TODO get rid of this timer
        // We give the backend some time to complete
        // the registration and setup the database.
        //
        // Intended behaviour: if we do not receive a
        // SetupWorker.setupChanged signal in the
        // meantime, we abort forcefully.
        // Problem: the signal is sent correctly but
        // WF ends up showing three pages on top of
        // eachother (this, FatalErrorPage, Main).
        // Current solution: wait some time, then show
        // the main page. Pray everything works out.
        interval: 250 /* 3000 */
        running: false
        onTriggered: {
            /* mainWindow.showFatalError(
                        xx("The registration may have failed, or "+
                           "your Internet connection is slow.")) */
            showMainPage()
        }
    }

    Column {
        width: parent.width
        spacing: 2*Theme.paddingLarge

        TextField {
            id: codeField
            width: parent.width - 4*Theme.horizontalPageMargin
            anchors.horizontalCenter: parent.horizontalCenter
            inputMethodHints: Qt.ImhDigitsOnly | Qt.ImhNoPredictiveText
            validator: RegExpValidator{ regExp: /|[0-9]{3}-[0-9]{3}/;}
            //: verification code input label
            //% "Verification code"
            label: qsTrId("whisperfish-verify-code-input-label")
            //: verification code input placeholder
            //% "Code"
            placeholderText: qsTrId("whisperfish-verify-code-input-placeholder")
            horizontalAlignment: TextInput.AlignHCenter
            font.pixelSize: Theme.fontSizeLarge
            EnterKey.onClicked: parent.forceActiveFocus() // CONTINUE?

            property int oldLength: 0
            onTextChanged: {
                // insert/remove a dash after the first 3 digits
                if (text.length != 3) return
                if (text.length > oldLength) { // getting entered
                    text += '-'
                } else if (text.length < oldLength) { // getting deleted
                    text = text.slice(0, 2)
                }
                oldLength = text.length
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
                anchors.horizontalCenter: parent.horizontalCenter
            }

            // TODO add second button to resend verification code
        }
    }
}
