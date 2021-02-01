import QtQuick 2.5
import Sailfish.Silica 1.0
import "../components"
import "../js/countries.js" as CallingCodes

BlockingInfoPageBase {
    id: root
    pageTitle: "" // xx("Step 2")

    //: registration page title
    //% "Register"
    mainTitle: qsTrId("whisperfish-registration-title")

    //: registration prompt text
    //% "Enter the phone number you want to register with Signal."
    mainDescription: qsTrId("whisperfish-registration-message")

    property bool _canAccept: !numberField.errorHighlight &&
                              prefixCombo.currentIndex >= 0 &&
                              numberField.text.replace(/[- ]*/, '').trim() !== ''

    signal accept
    onAccept: {
        if (!_canAccept) return
        busy = true // we have to wait for the backend to prompt the next step
        var iso = prefixCombo.currentItem.iso
        SettingsBridge.stringSet("country_code", iso)
        if (iso === "") console.warn("registering without ISO country code")
        Prompt.phoneNumber(prefixCombo.currentItem.prefix+numberField.text)
    }

    signal _retry
    on_Retry: {
        // TODO give haptic feedback

        //: new registration prompt text asking to retry
        //% "Please retry with a valid phone number."
        mainDescription = qsTrId("whisperfish-registration-retry-message")
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

                //: label for combo box for selecting calling code (phone number prefix)
                //: important: translate as short as possible
                //% "Prefix"
                description: qsTrId("whisperfish-registration-phone-number-prefix")

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
                validator: RegExpValidator{ regExp: /|[- 0-9]{4,}/ }

                //: phone number input label
                //% "Phone number"
                label: qsTrId("whisperfish-registration-number-input-label")

                //: phone number input placeholder
                //% "Phone number"
                placeholderText: qsTrId("whisperfish-registration-number-input-placeholder")
                EnterKey.iconSource: _canAccept ?
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
                //% "Signal will call you with a 6-digit verification code. Please be ready to write it down."
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
            //: continue button label
            //% "Continue"
            text: qsTrId("whisperfish-continue-button-label")
            enabled: _canAccept && !busy
            onClicked: accept()
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
}
