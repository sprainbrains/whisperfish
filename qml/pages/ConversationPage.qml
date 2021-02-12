import QtQuick 2.6
import Sailfish.Silica 1.0
import "../delegates"
import "../components"

Page {
    id: root
    objectName: conversationPageName

    // Enable to focus the editor when the page is opened.
    // E.g. when starting a new chat.
    property bool editorFocus: false

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

    // Desired design:
    // - message view: full screen, below custom page header
    // - input field: anchored at the bottom, transparent background,
    //   visible when the view is at the bottom (latest message) and
    //   hidden while scrolling, becomes visible when scrolling down a
    //   little bit, always visible while the keyboard is open, not
    //   visible during the quick scroll animation
    //
    // Implementation:
    // The message view is anchored below the page header and extends
    // to the bottom of the page. It has an empty header at the bottom
    // (because it is inverted). A OpacityRampEffect hides the message
    // view's contents below the header when it is shown. This is
    // necessary because \c{clip: true} does not clip the view below
    // the header.
    // The real input field is defined outside the view, thus it is not
    // affected by the transparency effect. It is anchored at the bottom
    // of the page, with its bottom margin bound to the list view header's
    // position.
    //
    // Note: to make the input field always visible (sticky), simply
    // set \c{messageView.anchors.bottom = inputField.anchors.top}.
    // Then remove the fake header item and the opacity ramp effect.

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
        header: Item {
            width: messages.width
            height: headerArea.height
        }

        onReplyTriggered: {
            // TODO textInput.replyToMessage(index, modelData)
            textInput.text = '> '+modelData.message.replace(/\n/g, '\n> ')+'\n'
            textInput.forceEditorFocus(true)
        }
    }

    OpacityRampEffect {
        sourceItem: messages
        direction: OpacityRamp.TopToBottom
        slope: messages.height
        offset: 1-((root.height-headerArea.y)/messages.height)
        enabled: headerArea.y < root.height &&
                 !messages.quickScrollAnimating &&
                 !messages.menuOpen
    }

    Item {
        id: headerArea
        width: parent.width
        height: textInput.height
        opacity: messages.menuOpen ? 0.0 : 1.0
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

        Behavior on opacity { FadeAnimator { duration: 100 } }

        ChatTextInput {
            id: textInput
            width: parent.width
            anchors.bottom: parent.bottom
            enablePersonalizedPlaceholder: messages.count === 0 && !MessageModel.group
            placeholderContactName: MessageModel.peerName
            editor.focus: root.editorFocus
            showSeparator: !messages.atYEnd

            onSendMessage: {
                // TODO This should be handled completely in the backend.
                // TODO Support multiple attachments in the backend.
                var sid = 0
                if (attachments.length > 0) {
                    sid = MessageModel.createMessage(MessageModel.peerTel, text, '',
                                                     attachments[0], true)
                } else {
                    sid = MessageModel.createMessage(MessageModel.peerTel, text,
                                                     '', '', true)
                }

                // update session model
                if(sid > 0) SessionModel.add(sid, true)
            }
        }
    }
}
