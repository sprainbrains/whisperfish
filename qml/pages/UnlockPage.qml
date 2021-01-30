import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: root

    // we don't want users to return to the empty landing page
    backNavigation: false
    showNavigationIndicator: false

    function isValid() {
        if (!SetupWorker.registered || passwordField.errorHighlight){
            return false
        }
        return true
    }

    function attemptUnlock() {
        if (isValid()) {
            Prompt.password(passwordField.text)

            // TODO Until we have a way of knowing if the entered
            // password was correct, we use the timer to continue
            // to the main page if no password prompt interrupts it.
            waitThenUnlock.restart()
        }
    }

    Connections {  // TO BE REMOVED
        // TODO This receives a new password prompt if the
        // password was incorrect. We don't want to lose time,
        // though. We should receive a success signal so we know
        // when/if it is safe to continue.
        id: validationConnection
        target: Prompt
        onPromptPassword: {
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

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height

        Column {
            id: column
            width: parent.width
            spacing: 1.5*Theme.paddingLarge

            PageHeader {
                title: qsTr("Unlock")
            }

            Item {
                anchors.horizontalCenter: parent.horizontalCenter
                // a bit larger than BusyIndicatorSize.Large
                width: 1.2*Theme.itemSizeExtraLarge
                height: 1.2*Theme.itemSizeExtraLarge

                Image {
                    id: appIcon
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectFit
                    // TODO use a higher resolution source image (not SVG though, not supported)
                    source: "../../icons/86x86/harbour-whisperfish.png"
                    verticalAlignment: Image.AlignVCenter
                    opacity: waitingSpinner.running ? Theme.opacityLow : 1.0
                    Behavior on opacity { FadeAnimator { } }
                }

                BusyIndicator {
                    id: waitingSpinner
                    anchors.centerIn: parent
                    size: BusyIndicatorSize.Large
                    running: waitThenUnlock.running
                    opacity: running ? 1.0 : 0.0
                    Behavior on opacity { FadeAnimator { } }
                }
            }

            Column {
                width: parent.width - 4*Theme.horizontalPageMargin
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingSmall

                Label {
                    width: parent.width
                    text: qsTr("Whisperfish")
                    color: Theme.highlightColor
                    font.pixelSize: Theme.fontSizeLarge
                    horizontalAlignment: Text.AlignHCenter
                }

                Label {
                    width: parent.width
                    text: qsTr("Please enter your password to unlock your conversations.")
                    color: Theme.secondaryHighlightColor
                    wrapMode: Text.WordWrap
                    font.pixelSize: Theme.fontSizeMedium
                    horizontalAlignment: Text.AlignHCenter
                }
            }

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
                color: errorHighlight ? Theme.highlightColor : Theme.primaryColor
                focus: true

                EnterKey.iconSource: SetupWorker.registered ? "image://theme/icon-m-enter-accept" :
                                                              "image://theme/icon-m-enter-next"
                EnterKey.onClicked: attemptUnlock()
            }

            Button {
                text: qsTr("Unlock")
                enabled: !passwordField.errorHighlight
                onClicked: attemptUnlock()
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }
    }
}
