// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
//import Nemo.Thumbnailer 1.0
import "../components"

MessageDelegateBase {
    width: parent.width
    enableDebugLayer: false

    Column {
        width: childrenRect.width
        height: childrenRect.height
        spacing: Theme.paddingMedium

        LinkedLabel {
            id: messageLabel
            width: Math.min(implicitWidth, maxMessageWidth)
            text: hasText ? // TODO Also check for attachments (then no text is ok).
                      modelData.message :
                      //: Placeholder note if an empty message is encountered.
                      //% "this message is empty"
                      qsTrId("whisperfish-message-empty-note")
            wrapMode: Text.Wrap
            horizontalAlignment: outgoing ? Text.AlignRight : Text.AlignLeft // TODO make configurable
            color: hasText ?
                       (highlighted ? Theme.highlightColor :
                                      (outgoing ? Theme.highlightColor :
                                                  Theme.primaryColor)) :
                       (highlighted ? Theme.secondaryHighlightColor :
                                      Theme.secondaryColor)
            font.pixelSize: Theme.fontSizeSmall // TODO make configurable
        }

        Label {
            width: Math.max(implicitWidth+Theme.paddingMedium, messageLabel.width)
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
}
