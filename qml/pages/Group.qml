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
            if (lst[i] !== SetupWorker.localId) {
                // TODO localId is available but not used by the backend, i.e. always empty
                var name = ContactModel.name(lst[i])
                if (name === lst[i]) {
                    // Unknown contact
                    //: Unknown contact in group member list
                    //% "Unknown"
                    name = qsTrId("whisperfish-unknown-contact")
                }

                contactListModel.append({"contactId": lst[i], "name": name})
            }
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
            contentHeight: Theme.itemSizeMedium

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
