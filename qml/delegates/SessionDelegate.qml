import QtQuick 2.4
//import QtQuick.Layouts 1.1
import Sailfish.Silica 1.0
import QtGraphicalEffects 1.0
import "../components"

ListItem {
    id: delegate
    property string name: model.isGroup ? model.groupName : ContactModel.name(model.source)
    property string message: {
        var re = (_debugMode ? "[" + model.id + "] " : "")
        if (model.message !== '') {
            return re+=model.message
        } else if (model.hasAttachment) {
            //: Session contains an attachment label
            //% "Attachment"
            re+=qsTrId("whisperfish-session-has-attachment")
        }
        return re
    }

    // NOTE Qt.DefaultLocaleShortDate includes seconds and takes too much space
    //: Time format including only hours and minutes, not seconds
    //% "hh:mm"
    property string date: Qt.formatTime(_rawDate, qsTrId("whisperfish-time-format-hours-minutes"))
    property int unreadCount: model.unread // TODO investigate: appears to be only 1 or 0
    property bool isGroup: model.isGroup
    property bool pinned: false // TODO implement in model
    property string profilePicture: '' // TODO implement in model
    property bool markReceived: model.received // TODO investigate: not updated for new message
    property bool markSent: model.sent // TODO cf. markReceived

    property bool _debugMode: SettingsBridge.boolValue("debug_mode")
    property var _rawDate: new Date(model.timestamp)
    property bool _labelsHighlighted: highlighted || unreadCount > 0

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

    Item {
        anchors { fill: parent; leftMargin: Theme.horizontalPageMargin }

        ProfilePicture {
            id: profilePicContainer
            highlighted: delegate.highlighted
            labelsHighlighted: delegate._labelsHighlighted
            imageSource: profilePicture
            isGroup: delegate.isGroup
            showInfoMark: pinned
            infoMark.source: 'image://theme/icon-s-low-importance'
            infoMark.rotation: 30
            anchors {
                left: parent.left
                verticalCenter: parent.verticalCenter
            }
            onClicked: console.log("profile picture clicked: "+name)
            onPressAndHold: delegate.openMenu()
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
            text: name
        }

        Label {
            id: lowerLabel
            anchors {
                left: upperLabel.left; right: unreadBackground.left
                top: upperLabel.bottom; bottom: parent.bottom
            }
            maximumLineCount: 2
            wrapMode: Text.WordWrap
            color: highlighted ? Theme.secondaryHighlightColor :
                                 Theme.secondaryColor
            font.pixelSize: Theme.fontSizeExtraSmall
            text: message
            highlighted: _labelsHighlighted
            verticalAlignment: Text.AlignTop
            elide: Text.ElideRight
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
                source: markReceived ? "../../icons/icon-s-received.png" :
                                       (markSent ? "../../icons/icon-s-sent.png" : "")
                anchors.verticalCenter: parent.verticalCenter
                highlighted: _labelsHighlighted
                width: Theme.iconSizeSmall; height: width
            }

            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: date
                highlighted: _labelsHighlighted
                font.pixelSize: Theme.fontSizeExtraSmall
                color: highlighted ? (unreadCount > 0 ? Theme.highlightColor :
                                                        Theme.secondaryHighlightColor) :
                                     (unreadCount > 0 ? Theme.highlightColor :
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
            width: unreadCount === 0 ? 0 : unreadLabel.width+Theme.paddingSmall
            height: width
            radius: 20
            color: Theme.highlightDimmerColor
            opacity: Theme.opacityLow
        }

        Label {
            id: unreadLabel
            anchors.centerIn: unreadBackground
            height: 1.2*Theme.fontSizeSmall; width: height
            text: unreadCount > 0 ? unreadCount : ''
            font.pixelSize: Theme.fontSizeExtraSmall
            minimumPixelSize: Theme.fontSizeTiny
            fontSizeMode: Text.Fit
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            color: Theme.highlightColor
            highlighted: _labelsHighlighted
        }
    }

    Component {
        id: contextMenuComponent

        ContextMenu {
            id: menu
            /* MenuItem {
                text: unreadCount === 0 ?
                          //: Mark conversation as 'unread', even though it isn't
                          //% "Mark as unread"
                          qsTrId("whisperfish-session-mark-unread") :
                          //: Mark conversation as 'read', even though it isn't
                          //% "Mark as read"
                          qsTrId("whisperfish-session-mark-read")
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
