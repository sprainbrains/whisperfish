import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.TextLinking 1.0
import be.rubdos.whisperfish 1.0
import "../components"

Page {
    id: profilePage
    objectName: "profilePage"

    property string profilePicture: ""
    property int recipientId: -1

    property bool isOwnProfile: SetupWorker.uuid === recipient.uuid

    Recipient {
        id: recipient
        app: AppState
        recipientId: profilePage.recipientId
    }

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height

        RemorsePopup { id: remorse }

        PullDownMenu {
            MenuItem {
                //: Reset identity key menu item
                //% "Reset identity key"
                text: qsTrId("whisperfish-reset-identity-menu")
                visible: SettingsBridge.debug_mode
                onClicked: {
                    //: Reset identity key remorse message (past tense)
                    //% "Identity key reset"
                    remorse.execute(qsTrId("whisperfish-reset-identity-message"),
                        function() {
                            console.log("Resetting identity key: " + recipient.e164)
                            SessionModel.removeIdentities(recipientId)
                        })
                }
            }
            MenuItem {
                //: Reset secure session menu item
                //% "Reset Secure Session"
                text: qsTrId("whisperfish-reset-session-menu")
                visible: SettingsBridge.debug_mode
                onClicked: {
                    //: Reset secure session remorse message (past tense)
                    //% "Secure session reset"
                    remorse.execute(qsTrId("whisperfish-reset-session-message"),
                        function() {
                            console.log("Resetting secure session with " + recipient.e164)
                            MessageModel.endSession(recipientId)
                        })
                }
            }
            MenuItem {
                //: Refresh contact profile menu item
                //% "Refresh Signal profile"
                text: qsTrId("whisperfish-refresh-profile-menu")
                visible: SettingsBridge.debug_mode
                onClicked: {
                    ClientWorker.refresh_profile(recipientId)
                }
            }
            MenuItem {
                //: Show a peer's system contact page (menu item)
                //% "Show contact"
                text: qsTrId("whisperfish-show-contact-page-menu")
                enabled: !isOwnProfile && recipient.e164.length > 0
                visible: enabled
                // TODO maybe: replace with a custom link handler
                onClicked: phoneNumberLinker.linkActivated('tel:' + recipient.e164)
                LinkedText { id: phoneNumberLinker; visible: false }
            }
        }

        Column {
            id: column
            width: parent.width
            spacing: Theme.paddingLarge

            PageHeader {
                title: recipient.name
                description: recipient.about
            }

            ProfilePicture {
                height: 2*Theme.itemSizeLarge
                width: height
                highlighted: false
                labelsHighlighted: false
                imageSource: profilePage.profilePicture
                isGroup: true
                showInfoMark: true
                infoMarkSource: 'image://theme/icon-s-chat'
                infoMarkSize: 0.9*Theme.iconSizeSmallPlus
                infoMarkEmoji: recipient.emoji
                anchors.horizontalCenter: parent.horizontalCenter
                onClicked: {
                    // TODO Implement a new page derived from ViewImagePage for showing
                    //      profile pictures. A new action overlay at the bottom can provide
                    //      options to change or delete the profile picture.
                    //      Note: adding a PullDownMenu would be best but is not possible.
                    //      ViewImagePage relies on Flickable and breaks if used with SilicaFlickable,
                    //      but PullDownMenu requires a SilicaFlickable as parent.

                    //pageStack.push(Qt.resolvedUrl("ViewImagePage.qml"), { 'title': groupName, 'source': imageSource })

                    remorse.execute("Changing the avatar is not yet implemented.", function() {})
                }
            }

            SectionHeader {
                //: Verify safety numbers
                //% "Verify safety numbers"
                text: qsTrId("whisperfish-verify-contact-identity-title")
            }

            Button {
                //: Show fingerprint button
                //% "Show fingerprint"
                text: qsTrId("whisperfish-show-fingerprint")
                enabled: numericFingerprint.text.length === 0
                visible: enabled
                onClicked: {
                    if(recipient.sessionFingerprint) {
                        var pretty_fp = ""
                        for(var i = 1; i <= 12; ++i) {
                            pretty_fp += recipient.sessionFingerprint.slice(5*(i-1), (5*i))
                            if(i === 4 || i === 8) {
                                pretty_fp += "\n"
                            } else if(i < 12) {
                                pretty_fp += " "
                            }
                        }
                        numericFingerprint.text = pretty_fp
                    }
                }
                anchors.horizontalCenter: parent.horizontalCenter
            }

            Rectangle {
                id: fingerprintBox
                anchors.horizontalCenter: parent.horizontalCenter
                width: numericFingerprint.width + 2*Theme.paddingLarge
                height: numericFingerprint.height + 2*Theme.paddingLarge
                radius: Theme.paddingLarge
                color: Theme.rgba(Theme.highlightBackgroundColor, Theme.highlightBackgroundOpacity)
                visible: numericFingerprint.text.length > 0
                Label {
                    id: numericFingerprint
                    anchors.centerIn: parent
                    font.family: 'monospace'
                }
            }

            TextArea {
                id: fingerprintDirections
                anchors.horizontalCenter: parent.horizontalCenter
                readOnly: true
                font.pixelSize: Theme.fontSizeSmall
                width: parent.width
                text: isOwnProfile
                    //: Numeric fingerprint instructions for own profile
                    //% "If you wish to verify the security of your end-to-end encryption with someone else, compare the numbers above with the numbers on their device."
                    ? qsTrId("whisperfish-numeric-fingerprint-directions-for-own-profile")
                    //: Numeric fingerprint instructions
                    //% "If you wish to verify the security of your end-to-end encryption with %1, compare the numbers above with the numbers on their device."
                    : qsTrId("whisperfish-numeric-fingerprint-directions").arg(recipient.name)
            }
        }
    }
}
