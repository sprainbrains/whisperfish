// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

// TODO implement receiving shared contacts in backend
AttachmentItemBase {
    id: item
    icon: 'image://theme/icon-m-contact'
    onClicked: if (_effectiveEnableClick) pageStack.push('../../pages/ContactCardPage.qml', {vcfUrl: attach.data})

    Column {
        anchors {
            left: parent.left; right: parent.right
            verticalCenter: parent.verticalCenter
        }

        Label {
            // TODO show contact name from file; this requires parsing the file
            //: Placeholder shown as title for an attached contact in a message
            //% "Shared contact"
            highlighted: item.highlighted ? true : undefined
            text: qsTrId("whisperfish-attachment-preview-contact-title")
            width: parent.width - Theme.paddingSmall
            truncationMode: TruncationMode.Fade
        }
        Label {
            highlighted: item.highlighted ? true : undefined
            text: _hasAttach ? attach.data : ''
            color: highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor
            width: parent.width - Theme.paddingSmall
            font.pixelSize: Theme.fontSizeExtraSmall
            elide: Text.ElideMiddle
        }
    }
}
