// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
//import Nemo.Thumbnailer 1.0
import "../components"

MessageDelegateBase {
    id: root
    delegateContentWidth: column.width
    enableDebugLayer: false
    readonly property int maxMessageLength: 600 // in characters; TODO make configurable

    property real labelWidth: Math.min(Math.max(infoLabel.width+statusIcon.width,
                                                metrics.width) + Theme.paddingMedium,
                                       maxMessageWidth)
    property real expandedWidth: root.width - 2*Theme.horizontalPageMargin
    property string messageText: hasText && typeof modelData.message !== 'undefined' &&
                                 modelData.message.trim() !== "" ?
                                     modelData.message :
                                     //: Placeholder note if an empty message is encountered.
                                     //% "this message is empty"
                                     qsTrId("whisperfish-message-empty-note")
    property bool isEmpty: !hasText || modelData.message.trim() === ""
    property bool canExpand: !isEmpty && modelData.message.length > maxMessageLength

    property bool _expanded: false

    onClicked: {
        if (canExpand) {
            _expanded = !_expanded
            // We make sure the list item is visible immediately
            // after changing the state. If omitted, closing a very
            // long delegate would leave the view to be positionend
            // somewhere off - possibly destroyed, and expansionTimer
            // would not trigger.
            listView.positionViewAtIndex(index, ListView.Contain)
            expansionTimer.start()
        } else {
            showMenu()
        }
    }

    Timer {
        // This timer waits a moment until we can be mostly certain
        // that the expansion is finished. It then positions the delegate
        // at the top of the page, i.e. ListView.End because the view
        // is inverted. Without the timer, the view would jump around.
        // (There is a slight flickering which can't be avoided this way.)
        id: expansionTimer
        interval: 100
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
        width: _expanded ? expandedWidth : labelWidth
        height: childrenRect.height
        spacing: Theme.paddingMedium

        // TODO Sender name for groups
        // Number and nickname, or saved contact name

        LinkedLabel {
            // TODO We may have to replace LinkedLabel with a custom
            // implementation to be able to use custom icons for emojis.
            id: messageLabel
            wrapMode: Text.Wrap
            anchors { left: parent.left; right: parent.right }
            horizontalAlignment: outgoing ? Text.AlignRight : Text.AlignLeft // TODO make configurable
            color: isEmpty ?
                       (highlighted ? Theme.secondaryHighlightColor :
                                      (outgoing ? Theme.secondaryHighlightColor :
                                                  Theme.secondaryColor)) :
                       (highlighted ? Theme.highlightColor :
                                      (outgoing ? Theme.highlightColor :
                                                  Theme.primaryColor))
            font.pixelSize: Theme.fontSizeSmall // TODO make configurable
            defaultLinkActions: false
            onLinkActivated: {
                Qt.openUrlExternally(link)
            }
            /*onTextChanged: {
                if (_expanded) listView.positionViewAtIndex(index, ListView.End)
            }*/

            states: [
                State {
                    name: "default"; when: !_expanded
                    PropertyChanges {
                        target: messageLabel
                        plainText: messageText.substr(0, maxMessageLength) + (canExpand ? ' ...' : '')
                    }
                },
                State {
                    name: "expanded"; when: _expanded
                    PropertyChanges {
                        target: messageLabel
                        plainText: messageText
                    }
                }
            ]
        }

        Row {
            id: infoRow
            spacing: 0
            layoutDirection: outgoing ? Qt.RightToLeft : Qt.LeftToRight
            anchors { left: parent.left; right: parent.right }

            // TODO Add debug info
            // TODO Fix right margin for incoming messages

            HighlightImage {
                id: statusIcon
                visible: outgoing
                width: Theme.iconSizeSmall
                height: width
                color: infoLabel.color
                source: {
                    if (modelData.read) "../../icons/icon-s-read.png"
                    else if (modelData.received) "../../icons/icon-s-received.png"
                    else if (modelData.sent) "../../icons/icon-s-sent.png"
                    // TODO actually use 'queued' state in model
                    else if (modelData.queued) "../../icons/icon-s-queued.png"
                    // TODO implement 'failed' state in model
                    // TODO check if SFOS 4 has "image://theme/icon-s-blocked" (3.4 doesn't)
                    else if (modelData.failed) "../../icons/icon-s-failed.png"
                    // TODO If all states are implemented and used, then we should
                    // change the default state to 'failed'. Until then the default
                    // has to be 'queued' to prevent a new message's icon to jump
                    // from 'failed' to 'received'.
                    else "../../icons/icon-s-queued.png"
                }
            }

            Label {
                id: infoLabel
                text: modelData.timestamp ?
                          Format.formatDate(modelData.timestamp, Formatter.TimeValue) :
                          //: Placeholder note if a message doesn't have a timestamp (which must not happen).
                          //% "no time"
                          qsTrId("whisperfish-message-no-timestamp")
                horizontalAlignment: outgoing ? Text.AlignRight : Text.AlignLeft // TODO make configurable
                font.pixelSize: Theme.fontSizeExtraSmall // TODO make configurable
                color: outgoing ?
                           (highlighted ? Theme.secondaryHighlightColor :
                                          Theme.secondaryHighlightColor) :
                           (highlighted ? Theme.secondaryHighlightColor :
                                          Theme.secondaryColor)
            }

            Row {
                id: showMoreRow
                visible: canExpand
                spacing: Theme.paddingSmall
                layoutDirection: outgoing ? Qt.LeftToRight : Qt.RightToLeft
                width: parent.width - infoLabel.width - statusIcon.width

                Item {
                    width: (outgoing ? 0.5 : -1.5) * Theme.paddingMedium
                    height: 1
                }
                Label {
                    font.pixelSize: Theme.fontSizeExtraSmall
                    text: "\u2022 \u2022 \u2022" // three dots
                }
                Label {
                    text: _expanded ?
                              //: Hint for very long messages, while expanded
                              //% "show less"
                              qsTrId("whisperfish-message-show-less") :
                              //: Hint for very long messages, while not expanded
                              //% "show more"
                              qsTrId("whisperfish-message-show-more")
                    font.pixelSize: Theme.fontSizeExtraSmall
                }
            }
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
