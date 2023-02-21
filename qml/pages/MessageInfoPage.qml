import QtQuick 2.6
import Sailfish.Silica 1.0
import be.rubdos.whisperfish 1.0
import "../components"
import "../delegates"

Page {
    id: root
    objectName: "messageInfoPage"

    property var message

    // Proxy some more used properties
    readonly property bool outgoing: message.outgoing

    Reactions {
        id: reactions
        app: AppState
        messageId: message.id
    }

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
                    width: parent.width
                    sourceComponent: defaultMessageDelegate
                }

                Component {
                    id: defaultMessageDelegate
                    MessageDelegate {
                        id: messageDelegate
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
                value: message.sessionId
            }
            DetailItem {
                //: Label for timestamp of the message
                //% "Timestamp"
                label: qsTrId("whisperfish-message-timestamp")
                value: message.timestamp
            }
            SectionHeader {
                visible: reactions.count
                //: Reactions section header
                //% "Reactions"
                text: qsTrId("whisperfish-message-info-reactions")
            }
            ListView {
                id: emojiView
                width: parent.width
                height: childrenRect.height
                model: reactions.reactions
                delegate: ListItem {
                    width: parent.width
                    height: childrenRect.height
                    DetailItem {
                        label: model.name
                        value: model.reaction
                    }
                }
            }
        }
    }
}
