// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import ".."

// This component must be a child of MessageDelegate.
LinkedEmojiLabel {
    plainText: (typeof modelData !== 'undefined' && modelData.reactions !== 'undefined') ? modelData.reactions : ""
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
