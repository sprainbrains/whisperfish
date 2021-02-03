import QtQuick 2.4
//import QtQuick.Layouts 1.1
import Sailfish.Silica 1.0
import QtGraphicalEffects 1.0
import "../components"

ListItem {
    id: delegate
    // property var date: new Date(model.timestamp)

    readonly property var names: [
        'Jane Doe', 'Erika Mustermann-Gabler', 'John Doe',
        'Cleopâtre Mustermann-Sodoge', 'Les Schtroumpfs',
        'Alice', 'Bob']
    readonly property var unread: [
        0, 2, 0, 15, 0, 1, 0]
    readonly property var group: [
        false, false, false, false, true, false, false]
    // readonly property var dates: ['11:32', '8:15', 'yesterday',
    //    'yesterday', '30.01.21', '28.12.20', '01.01.95']
    readonly property var dates: [
        '21:32', '10:15', '8:11',
        '10:00', '11:01', '7:34', '7:40']
    readonly property var texts: [
        'Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean',
        "tomorrow's fine, yup", "okay",
        'Asset csystems BATF Blowpipe Soviet South Africa wire transfer. NSA event security Compsec spies Benelux',
        'Alice: but what if...?', 'no'
    ]
    readonly property var avatars: ['pic1.png', 'pic2.png', 'pic3.png',
        '', '', '', ''
    ]

    property string name: names[index]
    property string message: texts[index]
    property int unreadCount: unread[index]
    property bool isGroup: group[index]
    property string date: dates[index]
    property bool pinned: index === 0
    property string profilePicture: avatars[index]

    property bool labelsHighlighted: highlighted || unreadCount > 0

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

    Item {
        anchors { fill: parent; leftMargin: Theme.horizontalPageMargin }

        ProfilePicture {
            id: profilePicContainer
            highlighted: delegate.highlighted
            labelsHighlighted: delegate.labelsHighlighted
            imageSource: profilePicture
            isGroup: delegate.isGroup
            showInfoMark: pinned
            infoMark.source: 'image://theme/icon-s-low-importance'
            infoMark.rotation: 30
            anchors {
                left: parent.left
                verticalCenter: parent.verticalCenter
            }
        }

        Label {
            id: upperLabel
            anchors {
                top: parent.top; topMargin: Theme.paddingMedium
                left: profilePicContainer.right; leftMargin: Theme.paddingLarge
                right: timeLabel.left; rightMargin: Theme.paddingMedium
            }
            highlighted: labelsHighlighted
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
            highlighted: labelsHighlighted
            verticalAlignment: Text.AlignTop
            elide: Text.ElideRight
        }

        Label {
            id: timeLabel
            anchors {
                leftMargin: Theme.paddingSmall
                right: parent.right; rightMargin: Theme.horizontalPageMargin
                verticalCenter: upperLabel.verticalCenter
            }
            text: date
            highlighted: labelsHighlighted
            font.pixelSize: Theme.fontSizeExtraSmall
            color: highlighted ? (unreadCount > 0 ? Theme.highlightColor :
                                                    Theme.secondaryHighlightColor) :
                                 (unreadCount > 0 ? Theme.highlightColor :
                                                    Theme.secondaryColor)
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
            highlighted: labelsHighlighted
        }
    }

    Component {
        id: contextMenuComponent

        ContextMenu {
            id: menu
            MenuItem {
                text: unreadCount === 0 ?
                          //: Mark conversation as 'unread', even though it isn't
                          //% "Mark as unread"
                          qsTrId("whisperfish-session-mark-unread") :
                          //: Mark conversation as 'read', even though it isn't
                          //% "Mark as read"
                          qsTrId("whisperfish-session-mark-read")
            }
            MenuItem {
                text: pinned ?
                          //: 'Unpin' conversation from the top of the view
                          //% "Unpin"
                          qsTrId("whisperfish-session-pin-view") :
                          //: 'Pin' conversation to the top of the view
                          //% "Pin to top"
                          qsTrId("whisperfish-session-unpin-view")
            }
            MenuItem {
                //: Delete all messages from session menu
                //% "Delete conversation"
                text: qsTrId("whisperfish-session-delete")
                onClicked: remove(delegate)
            }
        }
    }
}
