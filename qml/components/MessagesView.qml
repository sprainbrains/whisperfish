/*
 * Copyright (C) 2012-2015 Jolla Ltd.
 *
 * The code in this file is distributed under multiple licenses, and as such,
 * may be used under any one of the following licenses:
 *
 *   - GNU General Public License as published by the Free Software Foundation;
 *     either version 2 of the License (see LICENSE.GPLv2 in the root directory
 *     for full terms), or (at your option) any later version.
 *   - GNU Lesser General Public License as published by the Free Software
 *     Foundation; either version 2.1 of the License (see LICENSE.LGPLv21 in the
 *     root directory for full terms), or (at your option) any later version.
 *   - Alternatively, if you have a commercial license agreement with Jolla Ltd,
 *     you may use the code under the terms of that license instead.
 *
 * You can visit <https://sailfishos.org/legal/> for more information
 */

/*
 * Modifications for Whisperfish:
 * SPDX-FileCopyrightText: 2021 Mirian Margiani
 * SPDX-License-Identifier: AGPL-3.0-or-later
 *
 */

import QtQuick 2.6
import Sailfish.Silica 1.0
import "../delegates"

SilicaListView {
    id: messagesView
    verticalLayoutDirection: ListView.BottomToTop
    quickScroll: true  // TODO how to only allow downwards?

    // TODO verify:
    // avoids resetting focus every time a row is added, which breaks text input
    currentIndex: -1

    // TODO verify: date->string is always ISO formatted?
    // TODO Use a custom property for sections. It should contain
    // at least 1) date, 2) unread boundary, 3) ...
    // This would allow us to show sticky sections.
    //     section.labelPositioning: ViewSection.InlineLabels
    //     -> change to ViewSection.CurrentLabelAtStart if
    //        messageList.moving for a certain time
    section.property: "timestamp"

    property bool menuOpen: false

    signal replyTriggered(var index, var modelData)

    delegate: Item {
        id: wrapper
        property string newerSection: ListView.previousSection
        property string olderSection: ListView.nextSection
        property bool atSectionBoundary: {
            // Section strings are ISO formatted timestamps.
            // E.g. '2021-02-07T22:00:01'
            ((olderSection.substr(0, 10) !== "" &&
             olderSection.substr(0, 10) !== ListView.section.substr(0, 10))
                    || model.index === ListView.view.count - 1) ?
                        true : false
        }
        property Item section // overrides the default section item
        property bool isServiceMessage: false // TODO implement in backend

        height: loader.y + loader.height
        width: parent.width

        ListView.onRemove: loader.item.animateRemoval(wrapper)
        onAtSectionBoundaryChanged: {
            if (atSectionBoundary) {
                section = sectionHeaderComponent.createObject(wrapper, {
                    'title': ListView.section.substr(0, 10) === '' ?
                                 (newerSection.substr(0, 10) === '' ?
                                      MessageModel.peerName : newerSection) :
                                 Format.formatDate(ListView.section, Formatter.DateFull)
                })
            } else {
                // We manually remove the section item if it does not
                // match our criteria. ListView.sections.criteria is not
                // versatile enough. Using a Loader as section.delegate is
                // not possible because we wouldn't have access to section data.
                section.destroy()
                section = null
            }
        }

        Loader {
            id: loader
            y: section ? section.y + section.height : 0
            width: parent.width

            // TODO We can choose the delegate based on message contents,
            // e.g. a different delegate for stickers. This will improve
            // performance. (Once we have multiple delegates...)
            sourceComponent: wrapper.isServiceMessage ?
                                 serviceMessageDelegate :
                                 defaultMessageDelegate
        }

        Component {
            id: defaultMessageDelegate
            MessageDelegate {
                // necessary to make implicit properties available
                modelData: model
                menu: messageContextMenu
                // set explicitly because attached properties are not available
                // inside the loaded component
                listView: messagesView
                replySignal: replyTriggered
            }
        }

        Component {
            id: serviceMessageDelegate
            ServiceMessageDelegate {
                // necessary to make implicit properties available
                modelData: model
            }
        }
    }

    Component {
        id: sectionHeaderComponent
        Column {
            id: sectionHeader
            property string title: ""
            spacing: 0
            width: parent.width

            Item { width: 1; height: Theme.paddingLarge }

            Label {
                width: parent.width - 4*Theme.horizontalPageMargin
                anchors.horizontalCenter: parent.horizontalCenter
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                color: Theme.highlightColor
                text: parent.title

                Separator {
                    anchors {
                        horizontalCenter: parent.horizontalCenter
                        top: parent.baseline; topMargin: 8
                    }
                    width: parent.width-2*Theme.horizontalPageMargin
                    horizontalAlignment: Qt.AlignHCenter
                    color: Theme.highlightColor
                }
            }

            Item { width: 1; height: Theme.paddingLarge }
        }
    }

    VerticalScrollDecorator { flickable: messagesView }

    function openAttachment(contentItem) {
        MessageModel.openAttachment(contentItem.modelData.index)
    }

    function remove(contentItem) {
        //: Deleting message remorse
        //% "Deleting"
        contentItem.remorseAction(qsTrId("whisperfish-delete-message"),
            function() {
                console.log("Delete message: "+contentItem.modelData.id)
                MessageModel.remove(contentItem.modelData.index)
            })
    }

    function resend(contentItem) {
        //: Resend message remorse
        //% "Resending"
        contentItem.remorseAction(qsTrId("whisperfish-resend-message"),
            function() {
                console.log("Resending message: "+contentItem.modelData.id)
                MessageModel.sendMessage(contentItem.modelData.id)
            })
    }

    function copy(contentItem) {
        Clipboard.text = contentItem.modelData.message
    }

    Component {
        // Having context menu and worker functions are defined outside
        // the delegate means less code in the delegate. This could help
        // performance...?
        id: messageContextMenu

        ContextMenu {
            id: menu
            width: parent ? parent.width : Screen.width

            onActiveChanged: {
                // We can safely assume only one context menu is open at a time.
                // We assume less safely that this is the only menu component that will
                // ever be available in a message view.
                if (active) messagesView.menuOpen = true
                else messagesView.menuOpen = false
            }

            MenuItem {
                //: Copy message menu item
                //% "Copy"
                text: qsTrId("whisperfish-copy-message-menu")
                visible: menu.parent && menu.parent.hasText
                onClicked: copy(menu.parent)
            }
            MenuItem {
                //: Open attachment message menu item
                //% "Open"
                text: qsTrId("whisperfish-open-message-menu")
                visible: menu.parent && menu.parent.modelData.hasAttachment
                onClicked: openAttachment(menu.parent)
            }
            MenuItem {
                //: Delete message menu item
                //% "Delete"
                text: qsTrId("whisperfish-delete-message-menu")
                visible: true
                onClicked: remove(menu.parent)
            }
            MenuItem {
                //: Resend message menu item
                //% "Resend"
                text: qsTrId("whisperfish-resend-message-menu")
                visible: menu.parent && menu.parent.modelData.queued
                onClicked: resend(menu.parent)
            }
        }
    }
}
