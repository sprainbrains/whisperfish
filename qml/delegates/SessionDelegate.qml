import QtQuick 2.6
import Sailfish.Silica 1.0
import "../components"

ListItem {
    id: delegate
    property string date: model.section === 'older' ?
                              Format.formatDate(model.timestamp, Formatter.DateMedium) :
                              Format.formatDate(model.timestamp, Formatter.TimeValue)
    property bool isGroup: model.isGroup
    property int unreadCount: 0 // TODO implement in model
    property bool isUnread: model.unread // TODO investigate: is this really a bool?
    property bool isNoteToSelf: false // TODO implement in model (#138), e.g. SettingsBridge.stringValue("tel") === model.source
    property bool pinned: false // TODO implement in model
    property bool archived: false // TODO implement in model
    property bool hasDraft: false // TODO implement in model (#178)
    property string draft: '' // TODO implement in model (#178)
    property string profilePicture: '' // TODO implement in model (#192)
    property bool isPreviewReceived: model.received // TODO investigate: not updated for new message (#151, #55?)
    property bool isPreviewSent: model.sent // TODO cf. isPreviewReceived (#151)
    property bool hasAttachment: model.hasAttachment
    property string name: model.isGroup ? model.groupName : ContactModel.name(model.source)
    property string message: {
        var re = (_debugMode ? "[" + model.id + "] " : "")
        if (model.message !== '') {
            return re+=model.message
        } else if (hasAttachment) {
            // TODO we could show an icon in front
            //: Session contains an attachment label
            //% "Attachment"
            re+=qsTrId("whisperfish-session-has-attachment")
        }
        return re
    }

    property bool _debugMode: SettingsBridge.boolValue("debug_mode")
    property bool _labelsHighlighted: highlighted || isUnread

    contentHeight: 3*Theme.fontSizeMedium+2*Theme.paddingMedium+2*Theme.paddingSmall
    menu: contextMenuComponent
    ListView.onRemove: animateRemoval(delegate)

    function remove(contentItem) {
        //: Delete all messages from session
        //% "Deleting all messages"
        contentItem.remorseAction(qsTrId("whisperfish-session-delete-all"),
            function() {
                console.log("Deleting all messages for session: "+model.id)
                SessionModel.remove(model.index)
            })
    }

    function toggleReadState() {
        // TODO implement in model
        console.warn("setting read/unread is not implemented yet")
    }

    function togglePinState() {
        // TODO implement in model
        console.warn("setting pinned/unpinned is not implemented yet")
    }

    function toggleArchivedState() {
        // TODO implement in model
        console.warn("setting archived/not archived is not implemented yet")
    }

    Item {
        anchors { fill: parent; leftMargin: Theme.horizontalPageMargin }

        ProfilePicture {
            id: profilePicContainer
            highlighted: delegate.highlighted
            labelsHighlighted: delegate._labelsHighlighted
            imageSource: profilePicture
            isGroup: delegate.isGroup
            showInfoMark: pinned || archived || hasDraft || isNoteToSelf || hasAttachment
            infoMark.source: {
                if (hasDraft) 'image://theme/icon-s-edit'
                else if (isNoteToSelf) 'image://theme/icon-s-retweet' // task|secure|retweet
                else if (pinned) 'image://theme/icon-s-low-importance'
                else if (archived) 'image://theme/icon-s-time'
                else if (hasAttachment) 'image://theme/icon-s-attach'
                else ''
            }
            infoMark.rotation: {
                if (pinned) 30
                else if (hasDraft) -90
                else 0
            }
            anchors {
                left: parent.left
                verticalCenter: parent.verticalCenter
            }
            onPressAndHold: delegate.openMenu()
            onClicked: {
                MessageModel.load(model.id, ContactModel.name(model.source))
                if (isGroup) {
                    // TODO fixme: the group page has the group's name as header
                    //      but doesn't show any members
                    pageStack.push(Qt.resolvedUrl("../pages/Group.qml"))
                } else {
                    pageStack.push(Qt.resolvedUrl("../pages/VerifyIdentity.qml"))
                }
            }
        }

        Label {
            id: upperLabel
            anchors {
                top: parent.top; topMargin: Theme.paddingMedium
                left: profilePicContainer.right; leftMargin: Theme.paddingLarge
                right: timeLabel.left; rightMargin: Theme.paddingMedium
            }
            highlighted: _labelsHighlighted
            maximumLineCount: 1
            truncationMode: TruncationMode.Fade
            text: isNoteToSelf ?
                      //: Name of the conversation with one's own number
                      //% "Note to self"
                      qsTrId("whisperfish-session-note-to-self") :
                      name
        }

        LinkedEmojiLabel {
            id: lowerLabel
            anchors {
                left: upperLabel.left; right: unreadBackground.left
                top: upperLabel.bottom; bottom: parent.bottom
            }
            wrapMode: Text.WrapAnywhere
            maximumLineCount: 2
            enableElide: Text.ElideRight
            color: highlighted ? Theme.secondaryHighlightColor :
                                 Theme.secondaryColor
            font.pixelSize: Theme.fontSizeExtraSmall
            plainText: hasDraft ?
                      //: Message preview for a saved, unsent message
                      //% "Draft: %1"
                      qsTrId("whisperfish-message-preview-draft").arg(draft) :
                      message
            highlighted: _labelsHighlighted
            verticalAlignment: Text.AlignTop
        }

        Row {
            id: timeLabel
            spacing: Theme.paddingSmall
            anchors {
                leftMargin: Theme.paddingSmall
                right: parent.right; rightMargin: Theme.horizontalPageMargin
                verticalCenter: upperLabel.verticalCenter
            }

            HighlightImage {
                source: isPreviewReceived
                        ? "../../icons/icon-s-received.png" :
                          (isPreviewSent ? "../../icons/icon-s-sent.png" : "")
                color: Theme.primaryColor
                anchors.verticalCenter: parent.verticalCenter
                highlighted: _labelsHighlighted
                width: Theme.iconSizeSmall; height: width
            }

            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: date
                highlighted: _labelsHighlighted
                font.pixelSize: Theme.fontSizeExtraSmall
                color: highlighted ? (isUnread ? Theme.highlightColor :
                                                 Theme.secondaryHighlightColor) :
                                     (isUnread ? Theme.highlightColor :
                                                 Theme.secondaryColor)
            }
        }

        Rectangle {
            id: unreadBackground
            anchors {
                leftMargin: Theme.paddingSmall
                right: parent.right; rightMargin: Theme.horizontalPageMargin
                verticalCenter: lowerLabel.verticalCenter
            }
            visible: isUnread && unreadCount > 0
            width: isUnread ? unreadLabel.width+Theme.paddingSmall : 0
            height: width
            radius: 20
            color: profilePicContainer.profileBackgroundColor
        }

        Label {
            id: unreadLabel
            anchors.centerIn: unreadBackground
            height: 1.2*Theme.fontSizeSmall; width: height
            visible: isUnread && unreadCount > 0
            text: isUnread ? (unreadCount > 0 ? unreadCount : ' ') : ''
            font.pixelSize: Theme.fontSizeExtraSmall
            minimumPixelSize: Theme.fontSizeTiny
            fontSizeMode: Text.Fit
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            color: Theme.highlightColor
            highlighted: _labelsHighlighted
        }

        GlassItem {
            visible: isUnread
            color: Theme.highlightColor
            falloffRadius: 0.16
            radius: 0.15
            anchors {
                left: parent.left
                leftMargin: (width / -2) - Theme.horizontalPageMargin
                verticalCenter: parent.verticalCenter
            }
        }
    }

    Component {
        id: contextMenuComponent

        ContextMenu {
            id: menu
            /* MenuItem {
                text: isUnread ?
                          //: Mark conversation as 'read', even though it isn't
                          //% "Mark as read"
                          qsTrId("whisperfish-session-mark-read") :
                          //: Mark conversation as 'unread', even though it isn't
                          //% "Mark as unread"
                          qsTrId("whisperfish-session-mark-unread")
                onClicked: toggleReadState()
            }
            MenuItem {
                text: pinned ?
                          //: 'Unpin' conversation from the top of the view
                          //% "Unpin"
                          qsTrId("whisperfish-session-pin-view") :
                          //: 'Pin' conversation to the top of the view
                          //% "Pin to top"
                          qsTrId("whisperfish-session-unpin-view")
                onClicked: togglePinState()
            }
            MenuItem {
                text: archived ?
                          //: Show hidden messages again
                          //% "Un-archive conversation"
                          qsTrId("whisperfish-session-unarchive") :
                          //: Hide all messages from session menu
                          //% "Archive conversation"
                          qsTrId("whisperfish-session-archive")
                onClicked: toggleArchivedState()
            } */
            MenuItem {
                //: Delete all messages from session menu
                //% "Delete conversation"
                text: qsTrId("whisperfish-session-delete")
                onClicked: remove(delegate)
            }
        }
    }
}
