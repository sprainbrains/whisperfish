import QtQuick 2.2
import Sailfish.Silica 1.0
import "../delegates"

Page {
    id: conversation
    objectName: "conversation"
    property bool editorFocus

    property bool isGroup: MessageModel.group
    property var contact: isGroup ? null : resolvePeopleModel.personByPhoneNumber(MessageModel.peerTel, true)
    property string conversationName: isGroup ? MessageModel.peerName : (contact ? contact.displayLabel : MessageModel.peerTel)

    onStatusChanged: {
        if(status == PageStatus.Active) {
            if(MessageModel.group) {
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
            title: conversationName
            description:{
                // Attempt to display group member names
                if (isGroup) {
                    // XXX code duplication with Group.qml
                    var members = [];
                    var lst = MessageModel.groupMembers.split(",");
                    for(var i = 0; i < lst.length; i++) {
                        if(lst[i] != SetupWorker.localId) {
                            var member = resolvePeopleModel.personByPhoneNumber(lst[i], true);
                            if (!member) {
                                members.push(lst[i]);
                            } else {
                                members.push(member.displayLabel);
                            }
                        }
                    }
                    return members.join(", ");
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

            anchors {
                left: parent.left
                right: parent.right
            }

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
                    contactName: conversationName
                    enabled: true
                    editorFocus: conversation.editorFocus

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
