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

        PageHeader {
            id: pageHeader
            title:  MessageModel.peerName
            description:{
                // Attempt to display group member names
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
                else return (pageHeader.title == MessageModel.peerTel ? "" : MessageModel.peerTel)
            }
        }

        // https://together.jolla.com/question/196054/dialogheaderextracontent-read-only/
        HighlightImage {
            // There's currenty no option to move the icon to the right of the header.
            // It does exist in the ConversationHeader of jolla-messages,
            // but that's a very far stretch to import here.
            parent: pageHeader.extraContent
            id: pageHeaderImageIcon
            anchors.verticalCenter: parent.verticalCenter
            source: MessageModel.group ? "image://theme/icon-m-chat" : "image://theme/icon-m-contact"
        }

        MessagesView {
            id: messages
            focus: true
            height: parent.height - pageHeader.height
            contentHeight: height
            anchors { left: parent.left; right: parent.right }
            model: MessageModel

            // Use a placeholder for the ChatTextInput to avoid re-creating the input
            header: Item {
                width: messages.width
                height: textInput.height
            }

            Column {
                id: headerArea
                y: messages.headerItem.y
                parent: messages.contentItem
                width: parent.width

                WFChatTextInput {
                    id: textInput
                    width: parent.width
                    contactName: MessageModel.peerName
                    enabled: true
                    editorFocus: root.editorFocus

                    onSendMessage: {
                        var sid = MessageModel.createMessage(MessageModel.peerTel, text, "", attachmentPath, true)
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
