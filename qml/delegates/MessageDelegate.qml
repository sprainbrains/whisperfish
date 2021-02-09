// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
//import Nemo.Thumbnailer 1.0
import "../components"

MessageDelegateBase {
    width: parent.width
    enableDebugLayer: false
    readonly property int maxMessageLength: 300 // in characters; TODO make configurable

    property real labelWidth: Math.min(Math.max(infoLabel.implicitWidth+statusIcon.width,
                                                metrics.width) + Theme.paddingMedium,
                                       maxMessageWidth)
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
        } else {
            showMenu()
        }
    }

    TextMetrics {
        id: metrics
        text: messageLabel.plainText
        font: messageLabel.font
    }

    Column {
        width: labelWidth
        height: childrenRect.height
        spacing: Theme.paddingMedium

        // TODO Sender name for groups
        // Number and nickname, or saved contact name

        LinkedLabel {
            // TODO We may have to replace LinkedLabel with a custom
            // implementation to be able to use custom icons for emojis.
            id: messageLabel
            width: labelWidth
            plainText: messageText.substr(0, maxMessageLength) + (canExpand ? ' ...' : '')
            wrapMode: Text.Wrap
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

                Item { width: Theme.paddingMedium; height: 1 }
                Label {
                    font.pixelSize: Theme.fontSizeExtraSmall
                    text: "\u2022 \u2022 \u2022" // three dots
                }
                Label {
                    //% "show more"
                    text: qsTrId("whisperfish-message-show-more")
                    font.pixelSize: Theme.fontSizeExtraSmall
                }
            }
        }
    }

    states: [
        State {
            name: "outgoing"; when: outgoing
            AnchorChanges { target: messageLabel; anchors.right: parent.right }
        },
        State {
            name: "incoming"; when: !outgoing
            AnchorChanges { target: messageLabel; anchors.left: parent.left }
        }
    ]
}
