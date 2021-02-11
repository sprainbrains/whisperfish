import QtQuick 2.6
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

        ConversationPageHeader {
            id: pageHeader
            title:  MessageModel.peerName
            isGroup: MessageModel.group
            anchors.top: parent.top
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
            anchors {
                top: pageHeader.bottom;
                left: parent.left; right: parent.right
            }
            model: MessageModel
            clip: true // to prevent the view from flowing through the page header
            headerPositioning: ListView.PullBackHeader

            // Use a placeholder for the ChatTextInput to avoid re-creating the input
            // after it has been scrolled away. The input component is in the
            // header because the view is upside down.
            header: Item {
                width: messages.width
                height: headerArea.height
            }
        }

        OpacityRampEffect {
            sourceItem: messages
            direction: OpacityRamp.TopToBottom
            slope: messages.height //messages.height/(headerArea.height+Theme.paddingLarge)
            offset: 1-((root.height-headerArea.y/*+headerArea.height*/)/messages.height)
            enabled: headerArea.y < root.height && !messages.quickScrollAnimating
            onOffsetChanged: console.log(offset, slope)
        }

        Item {
            id: headerArea
            // y: messages.contentItem.y + messages.headerItem.y + pageHeader.height
            width: parent.width
            height: textInput.height + 2*Theme.paddingMedium
            anchors {
                bottom: parent.bottom
                bottomMargin: messages.quickScrollAnimating ?
                                  -height :
                                  (textInput.inputFieldFocused ?
                                       0 :
                                       parent.height - height -
                                       messages.contentItem.y -
                                       messages.headerItem.y -
                                       pageHeader.height)
            }

            WFChatTextInput {
                id: textInput
                width: parent.width
                contactName: MessageModel.peerName
                enabled: true
                editorFocus: root.editorFocus
                anchors.bottom: parent.bottom

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
