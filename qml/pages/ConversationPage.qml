import QtQuick 2.2
import Sailfish.Silica 1.0
import "../delegates"
import "../components"

Page {
    id: root
    objectName: conversationPageName

    property bool editorFocus

    onStatusChanged: {
        if (status == PageStatus.Active) {
            if (MessageModel.group) {
                pageStack.pushAttached(Qt.resolvedUrl("Group.qml"))
            } else {
                pageStack.pushAttached(Qt.resolvedUrl("VerifyIdentity.qml"))
            }
        }
    }

    Column {
        anchors.fill: parent

        ConversationPageHeader {
            id: pageHeader
            title:  MessageModel.peerName
            isGroup: MessageModel.group
            description: {
                // Attempt to display group member names
                // TODO This should be rewritten once the backend supports it (#223).
                if (MessageModel.group) {
                    var members = []
                    var lst = MessageModel.groupMembers.split(",")
                    for (var i = 0; i < lst.length; i++) {
                        if (lst[i] !== SetupWorker.localId) {
                            members.push(ContactModel.name(lst[i]))
                        }
                    }
                    return members.join(", ")
                }
                else return (MessageModel.peerName === MessageModel.peerTel ?
                                 "" : MessageModel.peerTel)
            }
        }

        MessagesView {
            id: messages
            focus: true
            height: parent.height - pageHeader.height
            contentHeight: height
            anchors { left: parent.left; right: parent.right }
            model: MessageModel

            // Use a placeholder for the ChatTextInput to avoid re-creating the input
            // after it has been scrolled away. The input component is in the
            // header because the view is upside down.
            header: Item {
                width: messages.width
                height: headerArea.height
            }

            Column {
                id: headerArea
                y: messages.headerItem.y
                parent: messages.contentItem
                width: parent.width
                height: childrenRect.height

                WFChatTextInput {
                    id: textInput
                    width: parent.width
                    contactName: MessageModel.peerName
                    enabled: true
                    editorFocus: root.editorFocus

                    onSendMessage: {
                        // TODO This should be handled completely in the backend.
                        var sid = MessageModel.createMessage(MessageModel.peerTel,
                                                             text, "", attachmentPath, true)
                        if(sid > 0) {
                            // Update session model
                            SessionModel.add(sid, true)
                        }
                    }
                }
            }
        }
    }
}
