// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
//import Nemo.Thumbnailer 1.0
import "../components"

MessageDelegateBase {
    width: parent.width
    enableDebugLayer: false

    property real labelWidth: Math.min(Math.max(infoLabel.implicitWidth, metrics.width) +
                                      Theme.paddingMedium, maxMessageWidth)
    property string messageText: hasText && typeof modelData.message !== 'undefined' &&
                                 modelData.message.trim() !== "" ?
                                     modelData.message :
                                     //: Placeholder note if an empty message is encountered.
                                     //% "this message is empty"
                                     qsTrId("whisperfish-message-empty-note")
    property bool isEmpty: !hasText || modelData.message.trim() === ""
    property bool canShowMore: !isEmpty && modelData.message.length > maxMessageLength

    TextMetrics {
        id: metrics
        text: messageText
        font: messageLabel.font
    }

    Column {
        width: labelWidth
        height: childrenRect.height
        spacing: Theme.paddingMedium

        LinkedLabel {
            id: messageLabel
            width: labelWidth
            plainText: messageText
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
        }

        Label {
            id: infoLabel
            width: labelWidth
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
    }

    states: [
        State {
            name: "outgoing"; when: outgoing
            AnchorChanges { target: messageLabel; anchors.right: parent.right }
            AnchorChanges { target: infoLabel; anchors.right: parent.right }
        },
        State {
            name: "incoming"; when: !outgoing
            AnchorChanges { target: messageLabel; anchors.left: parent.left }
            AnchorChanges { target: infoLabel; anchors.left: parent.left }
        }
    ]
}
