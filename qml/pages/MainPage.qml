import QtQuick 2.6
import Sailfish.Silica 1.0
import QtQml.Models 2.2
import "../delegates"

Page {
    id: main
    objectName: mainPageName

    SilicaFlickable {
        anchors.fill: parent

        VerticalScrollDecorator {}

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
            /*
            MenuItem {
                // TODO implement in backend
                //: Show list of bookmarked messages, menu item
                //% "Bookmarks"
                text: qsTrId("whisperfish-bookmarks-menu")
                visible: SessionModel.hasBookmarks
                onClicked: pageStack.push(Qt.resolvedUrl("BookmarksPage.qml"))
            }
            */
            /*
            MenuItem {
                // TODO implement in backend (#13)
                //: Show search field menu item
                //% "Search"
                text: qsTrId("whisperfish-search-menu")
                visible: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("SearchPage.qml"))
            }
            */
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

        PushUpMenu {
            MenuItem {
                visible: SessionModel.hasArchived // TODO implement in backend

                //: Menu item for showing archived conversations
                //% "Show archived conversations"
                property string showArchives: qsTrId("whisperfish-show-archived-menu")

                //: Menu item for returning to "inbox" from archived sessions
                //% "Return to conversations"
                property string showInbox: qsTrId("whisperfish-show-inbox-menu")

                text: visualSessionModel.filterOnGroup === "active"
                      ? showArchives
                      : showInbox
                onClicked: visualSessionModel.filterOnGroup = visualSessionModel.filterOnGroup === "archived"
                           ? "active"
                           : "archived"
            }
        }

        /**
         * Rust gives us the sessions sorted by [isPinned, timestamp] but in order to
         * split the messages into active and archived ones without rewriting/splitting
         * the SessionModel into two instances, DelegateModel can be used. This way
         * Rust side of things don't have to care if a message is archived or not.
         *
         * Basically "wrap" the SessionModel inside DelegateModel that handles this.
         */
        DelegateModel {
            id: visualSessionModel

            // Take the messages from "unsorted" group, and
            // push them either into "archived" or "artive"
            function sortToGroups() {
                var item
                while (unsortedItems.count > 0) {
                    item = unsortedItems.get(0)

                    if(item.model.isArchived) {
                        item.groups = "archived"
                    } else {
                        item.groups = "active"
                    }
                }
            }

            // "Update was requested."
            // Find the session, remove it from its group
            // by setting it in the unsorted group.
            // This triggers the placement logic.
            function clearItemGroup(sessionId) {
                var item
                if(filterOnGroup === "active") {
                    for(var i = 0; i < activeItems.count; i++) {
                        item = activeItems.get(i)
                        if(item.model.id === sessionId) {
                            item.groups = "unsorted"
                            return
                        }
                    }
                } else {
                    for(var i = 0; i < archivedItems.count; i++) {
                        item = archivedItems.get(i)
                        if(item.model.id === sessionId) {
                            item.groups = "unsorted"
                            return
                        }
                    }
                }
            }

            // Don't show all items by default
            items.includeByDefault: false

            // Which group to show: active or archived sessions
            filterOnGroup: "active"

            groups: [
                // All added sessions (or removed and re-added)
                // are inserted into "unsorted" by default.
                DelegateModelGroup {
                    id: unsortedItems
                    name: "unsorted"

                    // When the group contents change,
                    // automatically sort them into groups.
                    includeByDefault: true
                    onChanged: {
                        visualSessionModel.sortToGroups()
                    }
                },
                DelegateModelGroup {
                    id: activeItems
                    name: "active"
                    includeByDefault: false
                },
                DelegateModelGroup {
                    id: archivedItems
                    name: "archived"
                    includeByDefault: false
                }
            ]
            model: SessionModel
            delegate: SessionDelegate {
                onClicked: {
                    MessageModel.load(model.id, contact ? contact.displayLabel : model.source)
                    console.log("Activating session: "+model.id)
                    var contact = resolvePeopleModel.personByPhoneNumber(model.source)
                    pageStack.push(Qt.resolvedUrl("ConversationPage.qml"))
                }

                // On certain conditions, the session can request
                // the view to relocate itself.
                onRelocateItem: {
                    visualSessionModel.clearItemGroup(sessionId)
                }
            }
        }

        SilicaListView {
            id: sessionView
            model: visualSessionModel
            anchors.fill: parent
            spacing: 0
            header: PageHeader {
                title: "Whisperfish"

                //: Whisperfish subtitle for active conversations aka. "inbox"
                //% "Conversations"
                property string inboxSubtitle: qsTrId("whisperfish-subtitle-active-conversations")

                //: Whisperfish subtitle for active conversations aka. "inbox"
                //% "Archived conversations"
                property string archivedSubtitle: qsTrId("whisperfish-subtitle-archived-conversations")

                description: visualSessionModel.filterOnGroup === "active"
                             ? inboxSubtitle
                             : archivedSubtitle
            }

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

            height: contentHeight

            section {
                property: 'section'
                delegate: SectionHeader {
                    height: Theme.itemSizeExtraSmall
                    text: {
                        switch(section) {
                        case "pinned":
                            //: Session section label for pinned messages
                            //% "Pinned"
                            qsTrId("whisperfish-session-section-pinned")
                            break;
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
                            Qt.locale().dayName(parseInt(section), Locale.LongFormat)
                        }
                    }
                }
            }
        }
    }
}
