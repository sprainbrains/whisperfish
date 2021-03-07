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
    property DockedPanel activePanel: actionsPanel.open ? actionsPanel : panel

    property int _selectedCount: messages.selectedCount // proxy to avoid some costly lookups

    onStatusChanged: {
        if (status == PageStatus.Active) {
            SessionModel.markRead(MessageModel.sessionId)
            mainWindow.clearNotifications(MessageModel.sessionId)
            if (root.isGroup) {
                pageStack.pushAttached(Qt.resolvedUrl("GroupProfilePage.qml"))
            } else {
                pageStack.pushAttached(Qt.resolvedUrl("VerifyIdentity.qml"))
            }
        }
    }

    Connections {
        target: Qt.application
        onStateChanged: {
            if ((Qt.application.state === Qt.ApplicationActive) && (status === PageStatus.Active)) {
                SessionModel.markRead(MessageModel.sessionId)
                mainWindow.clearNotifications(MessageModel.sessionId)
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
        header: Item {
            height: activePanel.height; width: messages.width
            Behavior on height { NumberAnimation { duration: 150 } }
        }

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
        onQuoteClicked: {
            // TODO use message id instead of index
            jumpToMessage(quotedData.index)
        }
        onIsSelectingChanged: {
            if (isSelecting && !selectionBlocked) actionsPanel.show()
            else actionsPanel.hide()
        }
        onSelectedCountChanged: {
            if (selectedCount > 0 && !selectionBlocked) actionsPanel.show()
            else actionsPanel.hide()
        }
        onSelectionBlockedChanged: {
            if (selectionBlocked) actionsPanel.hide()
            else if (isSelecting) actionsPanel.show()
        }
    }

    OpacityRampEffect {
        sourceItem: messages
        direction: OpacityRamp.TopToBottom
        slope: sourceItem.height
        offset: 1-(activePanel.visibleSize/sourceItem.height)
        enabled: !sourceItem.quickScrollAnimating &&
                 !sourceItem.menuOpen
    }

    DockedPanel {
        id: panel
        background: null // transparent
        opacity: (actionsPanel.visibleSize > 0 || messages.menuOpen ||
                  messages.quickScrollAnimating) ? 0.0 : 1.0
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
                var firstAttachedPath = (attachments.length > 0 ? attachments[0].data : '')
                var sid = 0
                sid = MessageModel.createMessage(MessageModel.peerTel, text, '', firstAttachedPath, true)
                if (sid > 0) SessionModel.add(sid, true) // update session model

                // send remaining attachments in separate messages because the
                // backend does not support sending multiple attachments at once
                for (var i = 1; i < attachments.length; i++) {
                    sid = MessageModel.createMessage(MessageModel.peerTel, '', '', attachments[i].data, true)
                    if (sid > 0) SessionModel.add(sid, true) // update session model
                }
            }
        }
    }

    DockedPanel {
        id: actionsPanel
        background: null // transparent
        opacity: (messages.menuOpen || messages.quickScrollAnimating) ? 0.0 : 1.0
        width: parent.width
        height: actionsColumn.height + 2*Theme.horizontalPageMargin
        open: false
        dock: Dock.Bottom
        onOpenChanged: if (open) panel.hide()

        Behavior on opacity { FadeAnimator { duration: 80 } }

        Separator {
            opacity: messages.atYEnd ? 0.0 : Theme.opacityHigh
            color: Theme.secondaryHighlightColor
            horizontalAlignment: Qt.AlignHCenter
            anchors {
                left: parent.left; leftMargin: Theme.horizontalPageMargin
                right: parent.right; rightMargin: Theme.horizontalPageMargin
                top: parent.top
            }
            Behavior on opacity { FadeAnimator { } }
        }

        // ITEMS:
        // . = always visible
        // * = conditionally visible

        // -- CONTEXT MENU:
        // 0* resend        [if failed]
        // 1* react         [if not failed]
        // 2. copy
        // 3* forward       [if not failed]
        // 4. select · more

        // -- PANEL:
        // 1. clear selection
        // 2. copy
        // 3* info          [if only one selected]
        // 4. delete for me
        // 5. delete for all
        // 6* resend        [if at least one failed]

        Column {
            id: actionsColumn
            spacing: Theme.paddingLarge
            height: childrenRect.height
            anchors {
                verticalCenter: parent.verticalCenter
                left: parent.left; right: parent.right
            }

            InfoHintLabel {
                id: infoLabel
                //: Info label shown while selecting messages
                //% "%1 message(s) selected"
                defaultMessage: qsTrId("whisperfish-message-actions-info-label",
                                       _selectedCount).arg(messages.selectedCount)
            }

            // IMPORTANT:
            // - Both horizontal and vertical space may be very limited.
            //   There should never be more than two rows, and each row should
            //   contain at max. 4 icons at a time.
            // - Icons should always keep the same position so users can tap without looking.
            //   Entries may be hidden if they are at the sides and are seldomly used.
            //   Nothing should take the place of a hidden entry but there must not be any gaps.
            //   Entries that are conditionally unavailable should be deactivated, not hidden.
            //
            // TODO it may make sense to combine both rows into one in horizontal mode

            Row {
                spacing: Theme.paddingLarge
                anchors.horizontalCenter: parent.horizontalCenter
                IconButton {
                    width: Theme.itemSizeSmall; height: width
                    icon.source: "image://theme/icon-m-clear"
                    //: Message action description, shown if one or more messages are selected
                    //% "Clear selection"
                    onPressedChanged: infoLabel.toggleHint(
                                          qsTrId("whisperfish-message-action-clear-selection",
                                                 _selectedCount))
                    onClicked: messages.resetSelection()
                }
                IconButton {
                    width: Theme.itemSizeSmall; height: width
                    icon.source: "../../icons/icon-m-copy.png"
                    //: Message action description
                    //% "Copy %1 message(s)"
                    onPressedChanged: infoLabel.toggleHint(qsTrId("whisperfish-message-action-copy",
                                                                  _selectedCount).arg(_selectedCount))
                    onClicked: messages.messageAction(messages.copySelected)
                }
                IconButton {
                    width: Theme.itemSizeSmall; height: width
                    icon.source: "image://theme/icon-m-about"
                    //: Message action description (only available if n==1)
                    //% "Show message info"
                    onPressedChanged: infoLabel.toggleHint(qsTrId("whisperfish-message-action-info"))
                    enabled: _selectedCount === 1
                    onClicked: messages.messageAction(messages.showMessageInfo)
                }
            }
            Row {
                spacing: Theme.paddingLarge
                anchors.horizontalCenter: parent.horizontalCenter
                IconButton {
                    width: Theme.itemSizeSmall; height: width
                    icon.source: "image://theme/icon-m-delete"
                    //: Message action description
                    //% "Delete %1 message(s) for me"
                    onPressedChanged: infoLabel.toggleHint(
                                          qsTrId("whisperfish-message-action-delete-for-self",
                                                 _selectedCount).arg(_selectedCount))
                    onClicked: messages.messageAction(messages.deleteSelectedForSelf)
                }
                IconButton {
                    width: Theme.itemSizeSmall; height: width
                    icon.source: "../../icons/icon-m-delete-all.png"
                    //: Message action description
                    //% "Delete %1 message(s) for all"
                    onPressedChanged: infoLabel.toggleHint(
                                          qsTrId("whisperfish-message-action-delete-for-all",
                                                 _selectedCount).arg(_selectedCount))
                    onClicked: messages.messageAction(messages.deleteSelectedForAll)
                    enabled: false // TODO enable once implemented
                }

                // TODO find a way to count failed messages in the current selection
                IconButton {
                    width: visible ? Theme.itemSizeSmall : 0; height: width
                    icon.source: "image://theme/icon-m-refresh"
                    //: Message action description
                    //% "Retry sending (the) failed message(s)"
                    onPressedChanged: infoLabel.toggleHint(
                                          qsTrId("whisperfish-message-action-resend", _selectedCount))
                    visible: false // TODO show if at least one message is failed
                                   // NOTE this action should be *hidden* if it is not applicable
                    onClicked: messages.messageAction(messages.resendSelected)
                }
            }
        }
    }
}
