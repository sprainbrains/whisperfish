// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import ".."

Row {
    id: emojiItem
    property alias reactions: repeater.model
    anchors.margins: Theme.paddingMedium
    visible: repeater.count > 0
    property var color: isOutbound ?
            (highlighted ? Theme.secondaryHighlightColor :
                            Theme.secondaryHighlightColor) :
            (highlighted ? Theme.secondaryHighlightColor :
                            Theme.secondaryColor)
    Repeater {
        id: repeater
        model: emojiItem.model
        LinkedEmojiLabel {
            color: emojiItem.color
            plainText: model.reaction + (model.count > 1 ? model.count : '')
            font.pixelSize: Theme.fontSizeExtraSmall
        }
    }
}
