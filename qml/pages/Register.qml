import QtQuick 2.2
import Sailfish.Silica 1.0

Dialog {
    id: registerDialog
    objectName: "registerDialog"
    property string tel

    canAccept: !telField.errorHighlight

    onDone: {
        if (result == DialogResult.Accepted && !telField.errorHighlight) {
            tel = telField.text
            Prompt.phoneNumber(tel)
        }
    }

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        DialogHeader {
            //: Register accept text
            //% "Register"
            acceptText: qsTrId("whisperfish-register-accept")
        }

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            horizontalAlignment: Text.AlignLeft
            width: (parent ? parent.width : Screen.width) - Theme.paddingLarge * 2
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            //: Registration message
            //% "Enter the phone number you want to register with Signal."
            text: qsTrId("whisperfish-registration-message")
        }

        TextField {
            id: telField
            width: parent.width
            inputMethodHints: Qt.ImhDialableCharactersOnly | Qt.ImhNoPredictiveText
            validator: RegExpValidator{ regExp: /\+[0-9]+/;}
            //: Phone number input
            //% "International phone number"
            label: qsTrId("whisperfish-phone-number-input-label")
            //: Phone number placeholder
            //% "+18875550100"
            placeholderText: qsTrId("whisperfish-phone-number-input-placeholder")
            placeholderColor: Theme.highlightColor
            horizontalAlignment: TextInput.AlignLeft
            color: errorHighlight? "red" : Theme.primaryColor
            EnterKey.onClicked: parent.focus = true
        }

        IconTextSwitch {
            id: shareContacts
            anchors.horizontalCenter: parent.horizontalCenter
            //: Share contacts label
            //% "Share Contacts"
            text: qsTrId("whisperfish-share-contacts-label")
            //: Share contacts description
            //% "Allow Signal to use your local contact list, to find other Signal users."
            description: qsTrId("whisperfish-share-contacts-description")
            checked: SettingsBridge.boolValue("share_contacts")
            icon.source: "image://theme/icon-m-file-vcard"
            onCheckedChanged: {
                if(checked != SettingsBridge.boolValue("share_contacts")) {
                    SettingsBridge.boolSet("share_contacts", checked)
                }
            }
        }

        ComboBox {
            //: Verification method
            //% "Verification method"
            label: qsTrId("whisperfish-verification-method-label")

            menu: ContextMenu {
                MenuItem {
                    //: Text verification
                    //% "Use text verification"
                    text: qsTrId("whisperfish-use-text-verification")
                }
                MenuItem {
                    //: Voice verification
                    //% "Use voice verification"
                    text: qsTrId("whisperfish-use-voice-verification")
                }
            }

            onCurrentIndexChanged: {
                SetupWorker.useVoice = (currentIndex == 1)
            }
        }

        TextArea {
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width
            horizontalAlignment: TextEdit.Center
            readOnly: true
            visible: SetupWorker.useVoice
            //: Registration directions
            //% "Signal will call you with a 6-digit verification code. Please be ready to write this down."
            text: qsTrId("whisperfish-voice-registration-directions")
        }

        TextArea {
            anchors.horizontalCenter: parent.horizontalCenter
            width: parent.width
            horizontalAlignment: TextEdit.Center
            readOnly: true
            visible:  ! SetupWorker.useVoice
            //: Registration directions
            //% "Signal will text you a 6-digit verification code."
            text: qsTrId("whisperfish-text-registration-directions")
        }

    }
}
