// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

AttachmentItemBase {
    onClicked: if (_effectiveEnableClick) Qt.openUrlExternally(attach.data)

    Column {
        anchors {
            left: parent.left; right: parent.right
            verticalCenter: parent.verticalCenter
        }

        Label {
            text: _hasAttach ? lastPartOfPath(attach.data) : ''
            width: parent.width - Theme.paddingSmall
            elide: Text.ElideMiddle
        }
        Label {
            text: _hasAttach ? attach.data : ''
            color: Theme.secondaryColor
            width: parent.width - Theme.paddingSmall
            font.pixelSize: Theme.fontSizeExtraSmall
            elide: Text.ElideMiddle
        }
    }
}
