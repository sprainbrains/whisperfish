import QtQuick 2.2
import Sailfish.Silica 1.0

Dialog {
    id: addDeviceDialog
    objectName: "addDeviceDialog"

    onDone: {
        if (result == DialogResult.Accepted && urlField.acceptableInput) {
            if(urlField.text.length > 0) {
                addDevice(urlField.text)
            }
        }
    }

    signal addDevice(string tsurl)

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        DialogHeader {
            //: "Add" message, shown in the link device dialog
            //% "Add"
            acceptText: qsTrId("whisperfish-add-confirm")
        }

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            font.bold: true
            //: Add Device, shown as pull-down menu item
            //% "Add Device"
            text: qsTrId("whisperfish-add-device")
        }

        TextField {
            id: urlField
            width: parent.width
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhSensitiveData | Qt.ImhNoAutoUppercase | Qt.ImhPreferLowercase
            validator: RegExpValidator{ regExp: /(tsdevice|sgnl):\/\/?.*/;}
            //: Device URL, text input for pasting the QR-scanned code
            //% "Device URL"
            label: qsTrId("whisperfish-device-url")
            placeholderText: "sgnl://[...]"
            horizontalAlignment: TextInput.AlignLeft
            EnterKey.onClicked: parent.focus = true

            errorHighlight: !(urlField.text.length > 0 && urlField.acceptableInput)

            Component.onCompleted: {
                if(urlField.rightItem !== undefined) {
                    _urlFieldLoader.active = true
                    urlField.rightItem = _urlFieldLoader.item
                    urlField.errorHighlight = false
                }
            }

            Loader {
                id: _urlFieldLoader
                active: false
                sourceComponent: Image {
                    width: urlField.font.pixelSize
                    height: urlField.font.pixelSize
                    source: "image://theme/icon-s-checkmark?" + urlField.color
                    opacity: urlField.text.length > 0 && urlField.acceptableInput ? 1.0 : 0.01
                    Behavior on opacity { FadeAnimation {} }
                }
            }
        }

        Label {
            width: parent.width
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            //: Instructions on how to scan QR code for device linking
            //% "Install Signal Desktop. Use the CodeReader application to scan the QR code displayed on Signal Desktop and copy and paste the URL here."
            text: qsTrId("whisperfish-device-link-instructions")
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.highlightColor


            anchors {
                left: parent.left
                leftMargin: Theme.horizontalPageMargin
                right: parent.right
                rightMargin: Theme.horizontalPageMargin
            }
        }

    }
}
