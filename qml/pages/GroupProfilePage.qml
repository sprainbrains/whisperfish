import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.TextLinking 1.0
import be.rubdos.whisperfish 1.0
import "../components"

Page {
    id: groupProfile
    objectName: "groupProfilePage"

    // Group wallpapers/background are inherently un-sailfishy. We
    // should show them somewhere, somehow nonetheless - just not as
    // a background image. A group admin should be able to change it, too.
    /* property string groupWallpaper: '' */

    property var session: null;
    property var group: null;

    property bool groupV2: session.isGroupV2 // works until groupV3
    property string groupId: session.groupId
    property string groupName: session.groupName
    property string groupDescription: session.groupDescription ? session.groupDescription : ""

    readonly property string myUuid: SetupWorker.uuid
    readonly property string myPhone: SetupWorker.phoneNumber

    RemorsePopup { id: remorse }

    SilicaListView {
        id: flick
        anchors.fill: parent
        model: group.members
        header: Column {
            width: parent.width

            PageHeader {
                title: groupName
                description: !groupV2
                    //: Indicator for not yet updated groups
                    //% "Not updated to the new group format"
                    ? qsTrId("whisperfish-group-not-updated-to-groupv2")
                    : ""
            }

            ProfilePicture {
                id: groupAvatarItem
                height: 2*Theme.itemSizeLarge
                width: height
                highlighted: false
                labelsHighlighted: false
                imageSource: groupId !== undefined && groupId !== ''
                    ? SettingsBridge.avatar_dir + "/" + groupId
                    : ''
                isGroup: true
                showInfoMark: infoMarkSource !== ''
                infoMarkSource: groupV2 ? '' : 'image://theme/icon-s-filled-warning'
                infoMarkSize: 0.9*Theme.iconSizeSmallPlus
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

            Item {
                width: parent.width
                height: Theme.paddingLarge
            }

            Item {
                width: parent.width
                height: descriptionLabel.height
                Behavior on height { SmoothedAnimation { duration: 150 } }
                clip: true

                LinkedEmojiLabel {
                    id: descriptionLabel
                    property bool expanded: false

                    // TODO: the description should be editable if the user has the
                    //       appropriate permission (either admin, or all are allowed to edit)

                    x: Theme.horizontalPageMargin
                    width: parent.width-2*Theme.horizontalPageMargin
                    plainText: groupDescription
                    font.pixelSize: Theme.fontSizeSmall
                    // enableElide: Text.ElideRight -- no elide to enable dynamic height
                    // height: maximumLineCount*font.pixelSize
                    maximumLineCount: expanded ? 100000 : 5
                    emojiSizeMult: 1.0
                    horizontalAlignment: Text.AlignHCenter
                    color: expandDescriptionArea.pressed ?
                               Theme.secondaryHighlightColor :
                               Theme.secondaryColor
                    linkColor: color

                    MouseArea {
                        // no BackgroundItem to simplify placement, and we don't need the background
                        id: expandDescriptionArea
                        anchors.fill: parent
                        enabled: parent.truncated || parent.expanded
                        onClicked: parent.expanded = !parent.expanded
                    }
                }

                Label {
                    anchors {
                        bottom: descriptionLabel.bottom
                        right: descriptionLabel.right
                    }
                    font.pixelSize: Theme.fontSizeExtraSmall
                    text: "\u2022 \u2022 \u2022" // three dots
                    visible: descriptionLabel.truncated || descriptionLabel.expanded
                    color: expandDescriptionArea.pressed ?
                               Theme.highlightColor :
                               Theme.primaryColor
                }

                OpacityRampEffect {
                    direction: OpacityRamp.TopToBottom
                    offset: 0.5
                    slope: 2
                    sourceItem: descriptionLabel
                    enabled: descriptionLabel.truncated &&
                             !descriptionLabel.expanded
                }
            }

            Item { width: parent.width; height: Theme.paddingLarge }
        }

        VerticalScrollDecorator { flickable: flick }

        PullDownMenu {
            MenuItem {
                //: Refresh group menu item
                //% "Refresh group"
                text: qsTrId("whisperfish-group-refresh")
                onClicked: {
                    console.log("Refreshing group for session", session)
                    ClientWorker.refresh_group_v2(session.sessionId)
                }
            }
            MenuItem {
                //: Leave group menu item
                //% "Leave this group"
                text: qsTrId("whisperfish-group-leave-menu")
                onClicked: {
                    // TODO Leaving a group should *never* delete its messages.
                    //      Two different destructive actions should require two different
                    //      inputs and two confirmations.
                    //      Is it enough to remove the 'remove' line?
                    //: Leave group remorse message (past tense)
                    //% "Left group and deleted all messages"
                    remorse.execute(qsTrId("whisperfish-group-leave-remorse"),
                                    function() {
                                        console.log("Leaving group")
                                        MessageModel.leaveGroup()
                                        SessionModel.remove(session.sessionId)
                                        mainWindow.showMainPage()
                                    })
                }
            }
            MenuItem {
                // TODO implement in backend
                //: Create invite link menu item
                //% "Create invitation link"
                text: qsTrId("whisperfish-group-invite-link-menu")
                onClicked: remorse.execute("Changing group members is not yet implemented.", function() {})
            }
            MenuItem {
                // TODO implement in backend
                //: Add group member menu item
                //% "Add Member"
                text: qsTrId("whisperfish-group-add-member-menu")
                onClicked: remorse.execute("Changing group members is not yet implemented.", function() {})
            }
        }

        delegate: ListItem {
            id: item
            contentHeight: Theme.itemSizeMedium
            enabled: !isSelf

            property bool selfIsAdmin: false // TODO implement in backend
            property bool isVerified: false // TODO implement in backend;  model.isVerified
            property bool isSelf: model.uuid === myUuid
            property string profilePicture: getRecipientAvatar(model.e164, model.uuid)
            property string name: getRecipientName(model.e164, model.name, false)
            property bool isUnknownContact: name == model.e164

            // TODO Implement custom contact page for Whisperfish contacts
            onClicked:
                if(model.e164 != "") {
                    phonenumberLink.linkActivated('tel:' + model.e164)
                }

            menu: Component {
                ContextMenu {
                    MenuItem {
                        // TODO Implement a way to open a new chat with someone, or open
                        //      an existing chat. This requires better handling of sessions (#105, #183)
                        text: isSelf ?
                                  //: Menu item to open the conversation with oneself
                                  //% "Open Note to Self"
                                  qsTrId("whisperfish-group-member-menu-open-note-to-self") :
                                  //: Menu item to start a private chat with a group member
                                  //% "Message to %1"
                                  qsTrId("whisperfish-group-member-menu-direct-message").arg(
                                      isUnknownContact ? (model.e164 ? model.e164 : model.uuid) : name)
                        // TODO Remove the conditional once contact ids are no longer phone numbers,
                        //      and once profiles (nicknames) are implemented.
                        onClicked: remorse.execute("Directly opening a chat is not yet implemented.", function() {})
                    }
                    MenuItem {
                        //: Menu item to save a group member to the local address book
                        //% "Add to contacts"
                        text: qsTrId("whisperfish-group-member-menu-save-contact")
                        visible: isUnknownContact
                        onClicked: item.clicked(null) // show contact page
                    }
                    MenuItem {
                        //: Menu item to verify safety numbers with a group member
                        //% "Verify safety number"
                        text: qsTrId("whisperfish-group-member-menu-verify-fingerprint")
                        visible: !isVerified
                        onClicked: remorse.execute("Directly verifying the safety number is not yet implemented.", function() {})
                        // TODO We cannot open the verification page because we require access to the sessionId.
                        //      This should not be difficult anymore
                        //
                        // Not possible:
                        //      pageStack.push(Qt.resolvedUrl("../pages/VerifyIdentity.qml"), { sessionId:  })
                    }
                    MenuItem {
                        //: Menu item to remove a member from a group (requires admin privileges)
                        //% "Remove from this group"
                        text: qsTrId("whisperfish-group-member-menu-remove-from-group")
                        visible: selfIsAdmin
                        onClicked: remorse.execute("Changing group members is not yet implemented.", function() {})
                    }
                }
            }

            Row {
                width: parent.width - 2*Theme.horizontalPageMargin
                height: parent.height
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingLarge

                ProfilePicture {
                    highlighted: item.down
                    labelsHighlighted: highlighted
                    imageSource: item.profilePicture
                    isGroup: false // groups can't be members of groups
                    showInfoMark: false
                    anchors.verticalCenter: parent.verticalCenter
                    onPressAndHold: item.openMenu()
                    onClicked: item.clicked(null)
                }

                Column {
                    anchors {
                        verticalCenter: parent.verticalCenter
                        // where does the extra top padding come from?
                        verticalCenterOffset: -Theme.paddingSmall
                    }
                    Label {
                        font.pixelSize: Theme.fontSizeMedium
                        text: item.isSelf ?
                                  //: Title for the user's entry in a list of group members
                                  //% "You"
                                  qsTrId("whisperfish-group-member-name-self") :
                                  (item.isUnknownContact ?
                                      //: Unknown contact in group member list
                                      //% "Unknown"
                                      qsTrId("whisperfish-unknown-contact") :
                                      name)
                    }
                    LinkedText {
                        id: phonenumberLink
                        linkColor: color
                        color: item.down ? Theme.secondaryHighlightColor :
                                           Theme.secondaryColor
                        font.pixelSize: Theme.fontSizeSmall
                        plainText: model.e164
                    }
                }
            }
        }
    }
}
