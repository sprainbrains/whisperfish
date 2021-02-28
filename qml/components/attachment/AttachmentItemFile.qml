// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

AttachmentItemBase {
    id: item
    onClicked: if (_effectiveEnableClick) Qt.openUrlExternally(attach.data)

    Column {
        anchors {
            left: parent.left; right: parent.right
            verticalCenter: parent.verticalCenter
        }

        Label {
            highlighted: item.highlighted ? true : undefined
            text: _hasAttach ? lastPartOfPath(attach.data) : ''
            width: parent.width - Theme.paddingSmall
            elide: Text.ElideMiddle
        }
        Label {
            text: _hasAttach ? attach.data : ''
            highlighted: item.highlighted ? true : undefined
            color: highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor
            width: parent.width - Theme.paddingSmall
            font.pixelSize: Theme.fontSizeExtraSmall
            elide: Text.ElideMiddle
        }
    }
}
