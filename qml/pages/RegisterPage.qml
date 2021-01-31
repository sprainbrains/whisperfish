import QtQuick 2.5
import Sailfish.Silica 1.0
import "../components"
import "../js/countries.js" as CallingCodes

BlockingInfoPageBase {
    id: root
    pageTitle: "" // qsTr("Step 2")
    mainTitle: qsTr("Register")

    //: Registration message
    //% "Enter the phone number you want to register with Signal."
    mainDescription: qsTrId("whisperfish-registration-message")

    property bool _inputIsValid: !numberField.errorHighlight &&
                                 prefixCombo.currentIndex >= 0 &&
                                 numberField.text.replace(/[- ]*/, '').trim() !== ''

    signal accept
    onAccept: {
        if (!_inputIsValid) return
        busy = true // we have to wait for the backend to prompt the next step
        var iso = prefixCombo.currentItem.iso
        SettingsBridge.stringSet("country_code", iso)
        if (iso === "") console.warn("registering without ISO country code")
        Prompt.phoneNumber(prefixCombo.currentItem.prefix+numberField.text)
    }

    signal _retry
    on_Retry: {
        // TODO give haptic feedback
        mainDescription = qsTr("Please retry with a valid phone number.")
        busy = false
    }

    Connections {
        // We wait till the backend calls to continue.
        target: Prompt
        onPromptVerificationCode: pageStack.push(Qt.resolvedUrl("VerifyRegistrationPage.qml"))
        onPromptPhoneNumber: _retry()
    }

    Connections {
        target: SetupWorker
        onInvalidPhoneNumber: {
            console.log("invalid phone number")
            _retry()
        }
    }

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        Item {
            width: parent.width
            height: childrenRect.height

            ComboBox {
                id: prefixCombo
                anchors { left: parent.left; top: parent.top }
                width: Math.max(metrics.width+2*Theme.horizontalPageMargin+Theme.paddingSmall,
                                Theme.itemSizeExtraLarge)
                label: ""
                description: qsTr("Prefix")  // translate as short as possible
                currentIndex: -1
                value: currentIndex < 0 ?
                           '+xx' : currentItem.prefix
                menu: ContextMenu {
                    Repeater {
                        model: CallingCodes.c
                        MenuItem {
                            property string prefix: CallingCodes.c[index].p
                            property string name: CallingCodes.c[index].n
                            property string iso: CallingCodes.c[index].i
                            text: prefix + " - " + name + (iso ? " (%1)".arg(iso) : "")
                        }
                    }
                }

                TextMetrics {
                    id: metrics
                    font.pixelSize: Theme.fontSizeMedium
                    text: prefixCombo.value
                }
            }

            TextField {
                id: numberField
                anchors {
                    left: prefixCombo.right; leftMargin: 0
                    right: parent.right; rightMargin: Theme.horizontalPageMargin
                    verticalCenter: prefixCombo.verticalCenter
                }
                inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhDialableCharactersOnly
                validator: RegExpValidator{ regExp: /[- 0-9]{4,}/ }
                label: qsTr("Phone number")
                placeholderText: qsTr("Phone number")
                placeholderColor: color
                color: _inputIsValid ? Theme.primaryColor : Theme.highlightColor
                focus: true
                EnterKey.iconSource: _inputIsValid ?
                                         "image://theme/icon-m-enter-next" :
                                         "image://theme/icon-m-enter-close"
                EnterKey.onClicked: parent.forceActiveFocus()
            }
        }

        ComboBox {
            width: parent.width

            //: Verification method
            //% "Verification method"
            label: qsTrId("whisperfish-verification-method-label")

            //: Registration directions
            description: SetupWorker.useVoice
                //% "Signal will call you with a 6-digit verification code. Please be ready to write this down."
                ? qsTrId("whisperfish-voice-registration-directions")
                //% "Signal will text you a 6-digit verification code."
                : qsTrId("whisperfish-text-registration-directions")

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

        IconTextSwitch {
            id: shareContacts
            //: Share contacts label
            //% "Share Contacts"
            text: qsTrId("whisperfish-share-contacts-label")
            //: Share contacts description
            //% "Allow Signal to use your local contact list, to find other Signal users."
            description: qsTrId("whisperfish-share-contacts-description")
            checked: SettingsBridge.boolValue("share_contacts")
            icon.source: "image://theme/icon-m-file-vcard"
            onCheckedChanged: {
                if (checked !== SettingsBridge.boolValue("share_contacts")) {
                    SettingsBridge.boolSet("share_contacts", checked)
                }
            }
        }

        Button {
            text: qsTr("Continue")
            enabled: _inputIsValid && !busy
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
