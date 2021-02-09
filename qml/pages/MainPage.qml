import QtQuick 2.2
import Sailfish.Silica 1.0
import "../delegates"

Page {
    id: main
    objectName: mainPageName

    SilicaListView {
        id: sessionView
        model: SessionModel
        anchors.fill: parent
        spacing: Theme.paddingMedium
        footer: Item { width: parent.width; height: Theme.paddingMedium }

        PullDownMenu {
            // NOTE Make sure the pulley menu doesn't have more
            // than four entries. 'New group' and 'New message'
            // can be merged; 'About' and 'Settings' maybe too.
            // This makes room for 'Search' and 'Bookmarks'.

            MenuItem {
                //: About whisperfish menu item
                //% "About Whisperfish"
                text: qsTrId("whisperfish-about-menu")
                onClicked: pageStack.push(Qt.resolvedUrl("About.qml"))
            }
            MenuItem {
                //: Whisperfish settings menu item
                //% "Settings"
                text: qsTrId("whisperfish-settings-menu")
                visible: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("Settings.qml"))
            }
            /* MenuItem {
                // TODO implement in backend
                //: Show list of bookmarked messages, menu item
                //% "Bookmarks"
                text: qsTrId("whisperfish-bookmarks-menu")
                visible: SessionModel.hasBookmarks
                onClicked: pageStack.push(Qt.resolvedUrl("BookmarksPage.qml"))
            } */
            /* MenuItem {
                // TODO implement in backend (#13)
                //: Show search field menu item
                //% "Search"
                text: qsTrId("whisperfish-search-menu")
                visible: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("SearchPage.qml"))
            } */
            MenuItem {
                // TODO merge "new group" and "new message" as "new conversation"
                //: Whisperfish new group menu item
                //% "New Group"
                text: qsTrId("whisperfish-new-group-menu")
                visible: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("NewGroup.qml"))
            }
            MenuItem {
                //: Whisperfish new message menu item
                //% "New Message"
                text: qsTrId("whisperfish-new-message-menu")
                visible: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("NewMessage.qml"))
            }
        }

        /* PushUpMenu {
            MenuItem {
                visible: SessionModel.hasArchived // TODO implement in backend
                //: Menu item for showing archived conversations
                //% "Show archived conversations"
                text: qsTrId("whisperfish-show-archived-menu")
                onClicked: pageStack.push(Qt.resolvedUrl("ArchivePage.qml"))
            }
        } */

        VerticalScrollDecorator {}

        ViewPlaceholder {
            enabled: sessionView.count == 0
            // always show app name as placeholder, as the page
            // has not title which might be confusing
            text: "Whisperfish"
            hintText: {
                if (!SetupWorker.registered) {
                    //: Whisperfish registration required message
                    //% "Registration required"
                    qsTrId("whisperfish-registration-required-message")
                } else if (SetupWorker.locked) {
                    //: Whisperfish locked message
                    //% "Locked"
                    qsTrId("whisperfish-locked-message")
                } else {
                    //: No messages found, hint on what to do
                    //% "Pull down to start a new conversation."
                    qsTrId("whisperfish-no-messages-hint-text")
                }
            }
        }

        section {
            property: 'section'
            delegate: SectionHeader {
                height: Theme.itemSizeExtraSmall
                text: {
                    switch(section) {
                    case "today":
                        //: Session section label for today
                        //% "Today"
                        qsTrId("whisperfish-session-section-today")
                        break;
                    case "yesterday":
                        //: Session section label for yesterday
                        //% "Yesterday"
                        qsTrId("whisperfish-session-section-yesterday")
                        break;
                    case "older":
                        //: Session section label for older
                        //% "Older"
                        qsTrId("whisperfish-session-section-older")
                        break;
                    default:
                        // two days to one week ago
                        Qt.locale().standaloneDayName(parseInt(section), Locale.LongFormat)
                    }
                }
            }
        }

        delegate: SessionDelegate {
            onClicked: {
                console.log("Activating session: "+model.id)
                mainWindow.clearNotifications(model.id)
                pageStack.push(Qt.resolvedUrl("Conversation.qml"));
                if (model.unread) {
                    SessionModel.markRead(model.id)
                }
                MessageModel.load(model.id, ContactModel.name(model.source))
            }
        }
    }
}
