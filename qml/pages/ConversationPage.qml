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

    property bool isGroup: MessageModel.group
    property var contact: isGroup ? null : resolvePeopleModel.personByPhoneNumber(MessageModel.peerTel, true)
    property string conversationName: isGroup ? MessageModel.peerName : (contact ? contact.displayLabel : MessageModel.peerTel)

    onStatusChanged: {
        if (status == PageStatus.Active) {
            if (root.isGroup) {
                pageStack.pushAttached(Qt.resolvedUrl("GroupProfilePage.qml"))
            } else {
                pageStack.pushAttached(Qt.resolvedUrl("VerifyIdentity.qml"))
            }
        }
    }

    ConversationPageHeader {
        id: pageHeader
        title: conversationName
        isGroup: root.isGroup
        anchors.top: parent.top
        description: {
            // Attempt to display group member names
            // TODO This should be rewritten once the backend supports it (#223).
            if (root.isGroup) {
                // XXX code duplication with Group.qml
                var members = []
                var lst = MessageModel.groupMembers.split(",")
                for (var i = 0; i < lst.length; i++) {
                    if (lst[i] !== SetupWorker.localId) {
                        var member = resolvePeopleModel.personByPhoneNumber(lst[i], true)
                        if (!member) {
                            members.push(lst[i])
                        } else {
                            members.push(member.displayLabel)
                        }
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
    // the header, and a DockedPanel parented to the main window does not.
    // follow the page orientation.
    // The real input field is defined outside the view, thus it is not
    // affected by the transparency effect.

    MessagesView {
        id: messages
        focus: true
        height: parent.height - pageHeader.height
        contentHeight: height
        anchors {
            top: pageHeader.bottom; bottom: root.bottom
            left: parent.left; right: parent.right
        }
        model: MessageModel
        clip: true // to prevent the view from flowing through the page header
        headerPositioning: ListView.InlineHeader
        header: Item { height: panel.height; width: messages.width }

        onAtYEndChanged: panel.show()
        onMenuOpenChanged: panel.open = !messages.menuOpen
        onVerticalVelocityChanged: {
            if (panel.moving) return
            else if (verticalVelocity < 0) panel.hide()
            else if (verticalVelocity > 0) panel.show()
        }
        onReplyTriggered: {
            panel.show()
            textInput.setQuote(index, modelData)
            textInput.forceEditorFocus(true)
        }
    }

    OpacityRampEffect {
        sourceItem: messages
        direction: OpacityRamp.TopToBottom
        slope: sourceItem.height
        offset: 1-(panel.visibleSize/sourceItem.height)
        enabled: !sourceItem.quickScrollAnimating &&
                 !sourceItem.menuOpen
    }

    DockedPanel {
        id: panel
        background: null // transparent
        opacity: (messages.menuOpen || messages.quickScrollAnimating) ? 0.0 : 1.0
        width: parent.width
        height: textInput.height
        open: true
        dock: Dock.Bottom
        onHeightChanged: if (open) show()

        Behavior on opacity { FadeAnimator { duration: 80 } }

        ChatTextInput {
            id: textInput
            width: parent.width
            anchors.bottom: parent.bottom
            enablePersonalizedPlaceholder: messages.count === 0 && !root.isGroup
            placeholderContactName: conversationName
            editor.focus: root.editorFocus
            showSeparator: !messages.atYEnd || quotedMessageShown
            editor.onFocusChanged: if (editor.focus) panel.show()

            onQuotedMessageClicked: {
                // TODO use message id instead of index
                messages.jumpToMessage(index)
            }
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
