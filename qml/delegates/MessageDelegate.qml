// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
//import Nemo.Thumbnailer 1.0
import "../components"

MessageDelegateBase {
    id: root
    delegateContentWidth: column.width
    showExpand: !isEmpty && modelData.message.length > shortenThreshold

    readonly property int shortenThreshold: 600 // in characters
    readonly property int extraPageTreshold: 1500 // in characters

    property real labelWidth: Math.min(Math.max(metrics.width+messageLabel.emojiCount *
                                                messageLabel.font.pixelSize,
                                                minMessageWidth) +
                                       Theme.paddingMedium,
                                       maxMessageWidth)
    property string messageText: hasText && typeof modelData.message !== 'undefined' &&
                                 modelData.message.trim() !== "" ?
                                     modelData.message :
                                     //: Placeholder note if an empty message is encountered.
                                     //% "this message is empty"
                                     qsTrId("whisperfish-message-empty-note")
    property bool isEmpty: !hasText || modelData.message.trim() === ""
    property bool expandExtraPage: showExpand && modelData.message.length > extraPageTreshold

    // TODO Attachments with mimetype text/x-signal-plain have to be
    // treated as extra long messages.

    onClicked: {
        if (!showExpand) return

        if (expandExtraPage) {
            // TODO Cache the page object, so we can return to the
            // same scroll position where the user left the page.
            // It is not possible to re-use the returned object from pageStack.push().
            pageStack.push("../pages/ExpandedMessagePage.qml", {
                               'modelData': modelData,
                               'outgoing': outgoing
                           })
        } else {
            isExpanded = isExpanded
            // We make sure the list item is visible immediately
            // after changing the state. If omitted, closing a very
            // long delegate would leave the view to be positionend
            // somewhere off - possibly destroyed, and expansionTimer
            // would not trigger.
            listView.positionViewAtIndex(index, ListView.Contain)
            expansionTimer.start()
        }
    }

    Timer {
        // This timer waits a moment until we can be mostly certain
        // that the expansion is finished. It then positions the delegate
        // at the top of the page, i.e. ListView.End because the view
        // is inverted. Without the timer, the view would jump around.
        // TODO There is a some flickering which can't be avoided this way.
        //      (We need a better solution.)
        // TODO Sometimes jumping back fails...
        id: expansionTimer
        interval: isEmpty ? 0 : 5*modelData.message.length/shortenThreshold
        onTriggered: {
            listView.positionViewAtIndex(index, ListView.End)
        }
    }

    TextMetrics {
        id: metrics
        text: messageLabel.plainText
        font: messageLabel.font
    }

    Column {
        id: column
        width: isExpanded ? expandedWidth : labelWidth
        height: childrenRect.height
        spacing: Theme.paddingMedium

        LinkedEmojiLabel {
            id: messageLabel
            property bool emojiOnly: emojiCount > 0 && plainCharactersCount === 0 &&
                                     emojiCount <= 5 // treat long messages as text
            wrapMode: Text.Wrap
            anchors { left: parent.left; right: parent.right }
            horizontalAlignment: emojiOnly ? Text.AlignHCenter :
                                              (outgoing ? Text.AlignRight : Text.AlignLeft) // TODO make configurable
            color: isEmpty ?
                       (highlighted ? Theme.secondaryHighlightColor :
                                      (outgoing ? Theme.secondaryHighlightColor :
                                                  Theme.secondaryColor)) :
                       (highlighted ? Theme.highlightColor :
                                      (outgoing ? Theme.highlightColor :
                                                  Theme.primaryColor))
            linkColor: highlighted ? Theme.secondaryHighlightColor :
                                     Theme.secondaryColor
            enableCounts: true
            font.pixelSize: emojiOnly ?
                                (emojiCount <= 2 ? 1.5*Theme.fontSizeExtraLarge :
                                                   1.0*Theme.fontSizeExtraLarge) :
                                Theme.fontSizeSmall // TODO make configurable
            states: [
                State {
                    name: "default"; when: !isExpanded
                    PropertyChanges {
                        target: messageLabel
                        plainText: messageText.substr(0, shortenThreshold) + (showExpand ? ' ...' : '')
                    }
                },
                State {
                    name: "expanded"; when: isExpanded
                    PropertyChanges {
                        target: messageLabel
                        plainText: messageText
                    }
                }
            ]
        }

        states: [
            State {
                name: "outgoing"; when: outgoing
                AnchorChanges { target: column; anchors.right: parent.right }
            },
            State {
                name: "incoming"; when: !outgoing
                AnchorChanges { target: column; anchors.left: parent.left }
            }
        ]
    }
}
