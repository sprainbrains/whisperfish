import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.TextLinking 1.0

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
                //: Add group member menu item
                //% "Add Member"
                text: qsTrId("whisperfish-group-add-member-menu")
                onClicked: {
                    remorse.execute("Changing group members unimplemented", function() {})

                    return;
                    //: Add group member remorse message
                    //% "Adding %1 to group"
                    remorse.execute(qsTrId("whisperfish-group-add-member-remorse").arg(name),
                        function() {
                            // MessageModel.addMember(SetupWorker.localId, tel)
                        }
                    )
                }
            }
            MenuItem {
                //: Leave group menu item
                //% "Leave"
                text: qsTrId("whisperfish-group-leave-menu")
                onClicked: {
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
        }

        delegate: ListItem {   
            id: item
            contentHeight: Theme.itemSizeMedium
            enabled: !isSelf

            property bool selfIsAdmin: false // TODO implement in backend
            property bool isUnknownContact: model.isUnknown
            property bool isVerified: model.isVerified
            property bool isSelf: model.isSelf

            Column {
                id: column
                width: parent.width - 2*Theme.horizontalPageMargin
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingLarge

                Row {
                    spacing: Theme.paddingLarge
                    Column {
                        Row {
                            Label {
                                font.pixelSize: Theme.fontSizeMedium
                                text: name
                            }
                        }
                        Row {
                            LinkedText {
                                linkColor: Theme.highlightColor 
                                font.pixelSize: Theme.fontSizeExtraSmall
                                plainText: contactId
                            }
                        }
                        Row {
                            height: Theme.paddingLarge
                        }
                    }
                }
            }
        }
     }
}
