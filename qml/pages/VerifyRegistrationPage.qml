import QtQuick 2.5
import Sailfish.Silica 1.0
import "../components"

BlockingInfoPageBase {
    id: root
    pageTitle: "" // qsTr("Step 3")
    mainTitle: qsTr("Verify")
    mainDescription: qsTr("Please enter the code you received from Signal.")

    detailedDescription: SetupWorker.useVoice ?
                             qsTr("Signal should have called you with a a 6-digit "+
                                  "verification code. Please wait a moment, or "+
                                  "restart the process if you have not received a call.") :
                             qsTr("Signal should have sent you a 6-digit verification "+
                                  "code via text message. Please wait a moment, or "+
                                  "restart the process if you have not received a message.")
    squashDetails: true

    property bool _inputIsValid: !codeField.errorHighlight

    signal accept
    onAccept: {
        if (!_inputIsValid) return
        Prompt.verificationCode(codeField.text)
        busy = true // wait for the backend to prompt the next step

        // TODO We should receive a success/failure signal instead of
        // using this timer.
        waitThenUnlock.restart()
    }

    signal _retry
    on_Retry: {
        // TODO give haptic feedback
        mainDescription = qsTr("Please retry with a valid code.")
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
                        qsTr("The registration may have failed, or "+
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
            validator: RegExpValidator{ regExp: /[0-9]{6}/;}
            label: qsTr("Verification code")
            placeholderText: qsTr("Code")
            placeholderColor: Theme.highlightColor
            horizontalAlignment: TextInput.AlignHCenter
            font.pixelSize: Theme.fontSizeLarge
            color: _inputIsValid ? Theme.primaryColor : Theme.highlightColor
            EnterKey.onClicked: parent.forceActiveFocus() // CONTINUE?
        }

        Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: Theme.paddingMedium
            width: childrenRect.width
            height: childrenRect.height

            Button {
                text: qsTr("Continue")
                enabled: _inputIsValid
                onClicked: accept()
                anchors.horizontalCenter: parent.horizontalCenter
            }

            // TODO add second button to resend verification code
        }
    }
}
