// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import ".."

// This component must be a child of MessageDelegate.
Row {
    id: emojiRow
    spacing: 0
    layoutDirection: isOutbound ? Qt.RightToLeft : Qt.LeftToRight
    width: delegateContentWidth

    property real minContentWidth: delegateContentWidth

    LinkedEmojiLabel {
        text: typeof modelData !== 'undefined' && typeof modelData.reactions !== 'undefined' ? modelData.reactions : ""
        font.pixelSize: Theme.fontSizeExtraSmall
        color: isOutbound ?
                   (highlighted ? Theme.secondaryHighlightColor :
                                  Theme.secondaryHighlightColor) :
                   (highlighted ? Theme.secondaryHighlightColor :
                                  Theme.secondaryColor)
    }
}
