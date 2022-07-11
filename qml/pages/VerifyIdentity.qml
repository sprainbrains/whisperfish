import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.TextLinking 1.0

Page {
    id: verifyIdentity
    objectName: "verifyIdentity"

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height

        RemorsePopup { id: remorse }

        PullDownMenu {
            MenuItem {
                //: Reset identity key menu item
                //% "Reset identity key"
                text: qsTrId("whisperfish-reset-identity-menu")
                enabled: ! MessageModel.group && SettingsBridge.boolValue("debug_mode")
                onClicked: {
                    //: Reset identity key remorse message (past tense)
                    //% "Identity key reset"
                    remorse.execute(qsTrId("whisperfish-reset-identity-message"),
                        function() {
                            console.log("Resetting identity key: " + MessageModel.peerTel)
                            SessionModel.removeIdentities(MessageModel.sessionId)
                        })
                }
            }
            MenuItem {
                //: Reset secure session menu item
                //% "Reset Secure Session"
                text: qsTrId("whisperfish-reset-session-menu")
                enabled: ! MessageModel.group && SettingsBridge.boolValue("debug_mode")
                onClicked: {
                    //: Reset secure session remorse message (past tense)
                    //% "Secure session reset"
                    remorse.execute(qsTrId("whisperfish-reset-session-message"),
                        function() {
                            console.log("Resetting secure session: "+MessageModel.peerTel)
                            MessageModel.endSession(MessageModel.peerTel)
                        })
                }
            }
            MenuItem {
                //: Show a peer's system contact page (menu item)
                //% "Show contact"
                text: qsTrId("whisperfish-show-contact-page-menu")
                // TODO maybe: replace with a custom link handler
                onClicked: phoneNumberLinker.linkActivated('tel:'+MessageModel.peerTel)
                LinkedText { id: phoneNumberLinker; visible: false }
            }
        }

        Column {
            id: column
            width: parent.width
            spacing: Theme.paddingLarge

            PageHeader {
                title: MessageModel.peerName
            }

            SectionHeader {
                //: Verify safety numbers
                //% "Verify safety numbers"
                text: qsTrId("whisperfish-verify-contact-identity-title")
            }

            TextArea {
                id: numericFingerprint
                horizontalAlignment: TextEdit.Center
                readOnly: true
                width: parent.width
                font.family: 'monospace'
                text: MessageModel.numericFingerprint
            }

            TextArea {
                id: fingerprintDirections
                anchors.horizontalCenter: parent.horizontalCenter
                readOnly: true
                font.pixelSize: Theme.fontSizeSmall
                width: parent.width
                //: Numeric fingerprint instructions
                //% "If you wish to verify the security of your end-to-end encryption with %1, compare the numbers above with the numbers on their device."
                text: qsTrId("whisperfish-numeric-fingerprint-directions").arg(MessageModel.peerName)
            }
        }
    }
}
