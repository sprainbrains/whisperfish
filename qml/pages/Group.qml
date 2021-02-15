import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.TextLinking 1.0
import "../components"

Page {
    id: root

    ListModel {
        id: contactListModel
    }

    Component.onCompleted: {
        contactListModel.clear()
        var lst = MessageModel.groupMembers.split(",")
        for (var i = 0; i < lst.length; i++) {
            // TODO localId is available but not used by the backend, i.e. always empty
            var name = ContactModel.name(lst[i])
            var isUnknown = false // checked below
            var isVerified = false // TODO implement in backend

            // TODO We need a way localId is available but not used by the backend, i.e. always empty
            //      Related to #138. We need a way to check our own id.
            // TODO 'self' should always be the first entry in the list because the entry
            //      will not be clickable and act as a header.
            var isSelf = (lst[i] === SetupWorker.localId) // currently always false

            if (name === lst[i]) {
                // TODO Use nickname defined in the profile (#192)
                // Unknown contact
                //: Unknown contact in group member list
                //% "Unknown"
                name = qsTrId("whisperfish-unknown-contact")
                isUnknown = true
            }

            contactListModel.append({"contactId": lst[i],
                                        "name": name,
                                        "isUnknown": isUnknown,
                                        "isVerified": isVerified,
                                        "isSelf": isSelf,
                                    })
        }
    }

    RemorsePopup { id: remorse }

    SilicaListView {
        anchors.fill: parent
        model: contactListModel
        header: PageHeader { title: MessageModel.peerName }

        PullDownMenu {
            MenuItem {
                //: Leave group menu item
                //% "Leave this group"
                text: qsTrId("whisperfish-group-leave-menu")
                onClicked: {
                    // TODO Leaving a group should *never* delete its messages.
                    //      Two different destructive actions should require two different
                    //      inputs and two confirmations.
                    //      Is it enough to remove the 'removeById' line?
                    //: Leave group remorse message
                    //% "Leaving group and removing ALL messages!"
                    remorse.execute(qsTrId("whisperfish-group-leave-remorse"),
                        function() {
                            console.log("Leaving group")
                            MessageModel.leaveGroup()
                            SessionModel.removeById(MessageModel.sessionId)
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
            property bool isUnknownContact: model.isUnknown
            property bool isVerified: model.isVerified
            property bool isSelf: model.isSelf

            // TODO This is an ugly hack that relies on contactId being a phone number.
            //      - Remove if/when contacts move to UUIDs
            //      - Implement custom contact page for Whisperfish contacts
            onClicked: phonenumberLink.linkActivated('tel:'+contactId)

            Row {
                width: parent.width - 2*Theme.horizontalPageMargin
                height: parent.height
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingLarge

                ProfilePicture {
                    highlighted: item.down
                    labelsHighlighted: highlighted
                    imageSource: '' // TODO implement somewhere
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
                                  //: TODO
                                  //% "You"
                                  qsTrId("whisperfish-group-member-name-self") :
                                  name
                    }
                    LinkedText {
                        id: phonenumberLink
                        linkColor: color
                        color: item.down ? Theme.secondaryHighlightColor :
                                           Theme.secondaryColor
                        font.pixelSize: Theme.fontSizeSmall
                        plainText: contactId
                    }
                }
            }
        }
     }
}
