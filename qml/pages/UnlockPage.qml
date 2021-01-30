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

            // TODO Move this to a success handler in validationConnection.
            // Continue only if password is correct.
            // -- mainWindow.showMainPage()

            // TODO Until there's a success signal in rust, we return
            // to the landing page, wait a moment for any error,
            // and continue to the main page.
            pageStack.pop()
        }
    }

    /* Connections {
        // TODO This receives a new password prompt if the
        // password was incorrect. We don't want to lose time,
        // though. We should receive a success signal so we know
        // when/if it is safe to continue.
        id: validationConnection
        target: Prompt
        onPromptPassword: {
            passwordField.text = ''
            // TODO give haptic feedback
            passwordField.placeholderText = qsTr("Please try again")
        }
    } */

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

            Image {
                anchors.horizontalCenter: parent.horizontalCenter
                width: Theme.itemSizeExtraLarge
                height: Theme.itemSizeExtraLarge
                fillMode: Image.PreserveAspectFit
                source: "../../icons/86x86/harbour-whisperfish.png"
                verticalAlignment: Image.AlignVCenter
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
