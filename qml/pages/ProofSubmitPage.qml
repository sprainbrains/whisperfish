import QtQuick 2.5
import Sailfish.Silica 1.0
import Nemo.DBus 2.0
import "../components"

BlockingInfoPageBase {
    id: root
    pageTitle: "" // xx("Step 2")

    //: Signal has requested additional captcha page title
    //% "reCaptcha requested"
    mainTitle: qsTrId("whisperfish-captcha-requested-title")

    //: Signal has requested additional captcha description
    //% "Signal has requested additional capcha from you. Continue the captcha in order to restore ability to send messages."
    mainDescription: qsTrId("whisperfish-captcha-requested-message")

    property bool captchaReceived: false
    property string captchaToken

    DBusAdaptor {
        service: "be.rubdos.whisperfish"
        path: "/be/rubdos/whisperfish/captcha"
        iface: "be.rubdos.whisperfish.captcha"

        function handleCaptcha(captchaResponse) {
            console.log("handleCaptcha()")
            if(!captchaReceived) {
                captchaReceived = true
                mainWindow.activate()
                console.log("Submit token:", captchaToken)
                console.log("Submit captcha:",captchaResponse)
                //Prompt.proofCaptcha(captchaToken, captchaResponse)
                root.busy = false
                popTimer.restart()
            }
        }
    }

    Timer {
        id: captchaTimer
        interval: 100
        running: false
        repeat: false
        onTriggered: {
            captchaReceived = false
            Prompt.startCaptcha()
        }
    }

    Timer {
        id: popTimer
        interval: 100
        running: false
        repeat: false
        onTriggered: {
            pageStack.pop()
        }
    }

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        Button {
            text: (captchaReceived
                //: Done button label
                //% "Done"
                ? qsTrId("whisperfish-done-button-label")
                //: Continue button label
                //% "Continue"
                : qsTrId("whisperfish-continue-button-label")
            )
            enabled: !busy
            onClicked: {
                if (captchaReceived) {
                    popTimer.restart()
                    root.busy = true
                } else {
                    captchaTimer.restart()
                    root.busy = true
                }
            }
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
