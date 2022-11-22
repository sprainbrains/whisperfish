import QtQuick 2.6
import Sailfish.Silica 1.0
import "../components"
import "../delegates"

Page {
    id: root
    property var message

    // Proxy some more used properties
    readonly property bool outgoing: message.outgoing

    SilicaFlickable {
        id: silicaFlickable
        anchors.fill: parent

        contentHeight: contentColumn.height + Theme.paddingLarge

        VerticalScrollDecorator {
            flickable: silicaFlickable
        }

        Column {
            id: contentColumn
            anchors {
                top: parent.top
                left: parent.left
                right: parent.right
            }

            spacing: Theme.paddingMedium

            PageHeader {
                id: pageHeader
                //: Page title for message info/details page
                //% "Message Info"
                title: qsTrId("whisperfish-message-info-title")
            }

            Item {
                // TODO: Disable touches properly.
                // 'enabled: false' messes up visuals
                id: messageItem
                property bool atSectionBoundary: false
                property bool isServiceMessage: false

                height: loader.y + loader.height
                width: parent.width

                Loader {
                    id: loader
                    y: section ? section.y + section.height : 0
                    width: parent.width
                    sourceComponent: defaultMessageDelegate
                }

                Component {
                    id: defaultMessageDelegate
                    MessageDelegate {
                        modelData: message
                        //menu: messageContextMenu
                        // set explicitly because attached properties are not available
                        // inside the loaded component
                        showSender: true
                        // No menus here!
                        openMenuOnPressAndHold: false
                    }
                }
            }

            SectionHeader {
                //: Details section header
                //% "Details"
                text: qsTrId("whisperfish-message-info-details")
            }
            DetailItem {
                //: Label for id of the message (in database)
                //% "Message ID"
                label: qsTrId("whisperfish-message-message-id")
                value: message.id
            }
            DetailItem {
                //: Label for session id of the message (in database)
                //% "Session ID"
                label: qsTrId("whisperfish-message-session-id")
                value: message.sid
            }
            DetailItem {
                //: Label for timestamp of the message
                //% "Timestamp"
                label: qsTrId("whisperfish-message-session-id")
                value: message.timestamp
            }
            SectionHeader {
                visible: emojiLabel.visible
                //: Reactions section header
                //% "Reactions"
                text: qsTrId("whisperfish-message-info-reactions")
            }
            LinkedEmojiLabel {
                id: emojiLabel
                visible: plainText.length > 0
                anchors {
                    left: parent.left
                    right: parent.right
                    leftMargin: Theme.paddingLarge * 2
                }
                plainText: message.reactionsNamed
            }
        }
    }
}