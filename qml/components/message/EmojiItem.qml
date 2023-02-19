// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import ".."

// This component must be a child of MessageDelegate.
LinkedEmojiLabel {
    property var reactions: null

    Component.onCompleted: {
        if(reactions.count == 0)
            return;
        var emojis = reactions.groupedReactions
        var text = ''
        if (emojis) {
            for (var key in emojis) {
                text = text + key + " " + (emojis[key] > 1 ? (emojis[key] + " ") : '')
            }
        }
        emojiLabel.plainText = text
    }

    plainText: ''
    id: emojiLabel
    anchors.margins: Theme.paddingMedium
    visible: plainText.length > 0
    font.pixelSize: Theme.fontSizeExtraSmall
    color: isOutbound ?
                (highlighted ? Theme.secondaryHighlightColor :
                                Theme.secondaryHighlightColor) :
                (highlighted ? Theme.secondaryHighlightColor :
                                Theme.secondaryColor)
}
