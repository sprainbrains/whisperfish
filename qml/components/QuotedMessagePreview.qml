// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "attachment"

BackgroundItem {
    id: root
    // 'attachments' is expected as a list of objects: [{data: path, type: mimetype}, ...]
    property var messageData: null // required properties: message, source, outgoing, attachments
    property bool showCloseButton: true
    property bool showBackground: false
    property real contentPadding: Theme.paddingMedium

    property alias maximumWidth: senderNameLabel.maximumWidth
    property alias horizontalAlignment: textLabel.horizontalAlignment
    property alias backgroundItem: bgRect

    readonly property bool shown: (messageData !== null && visible)
    readonly property bool hasAttachments: false
    // readonly property bool hasAttachments: (
    //     (messageData.thumbsAttachments !== undefined ? messageData.thumbsAttachments.count : 0)
    //     + (messageData.detailAttachments !== undefined ? messageData.detailAttachments.count : 0)
    //     > 0)

    implicitWidth: shown ? Math.min(Math.max(senderNameLabel.implicitWidth+2*contentPadding,
                                             metrics.width), maximumWidth) : 0
    implicitHeight: shown ? Math.max(quoteColumn.height, attachThumb.height) : 0
    height: implicitHeight
    width: implicitWidth
    _backgroundColor: "transparent"

    signal closeClicked(var mouse)

    TextMetrics {
        id: metrics
        font: textLabel.font
        text: textLabel.plainText
    }

    HighlightImage {
        id: closeButton
        visible: shown && showCloseButton
        // HighlightImage with separate MouseArea instead of IconButton
        // for finer control over size and placement
        anchors {
            verticalCenter: parent.verticalCenter
            right: quoteColumn.left; rightMargin: Theme.paddingMedium
        }
        width: visible ? Theme.iconSizeSmallPlus : 0
        height: width
        horizontalAlignment: Image.AlignHCenter
        verticalAlignment: Image.AlignVCenter
        source: "../../icons/icon-s-close.png"
        highlighted: closeButtonArea.pressed || root.down

        MouseArea {
            id: closeButtonArea
            anchors.centerIn: parent
            width: 3*Theme.iconSizeSmall
            height: width
            onClicked: closeClicked(mouse)
        }
    }

    RoundedRect {
        id: bgRect
        visible: shown && showBackground
        color: down ? Theme.highlightBackgroundColor :
                      Theme.rgba(Theme.secondaryColor, Theme.opacityFaint)
        opacity: Theme.opacityFaint
        roundedCorners: allCorners
        anchors.fill: parent
        radius: Theme.paddingMedium
    }

    Column {
        id: quoteColumn
        visible: shown
        topPadding: padding-0.9*Theme.paddingSmall // remove excessive top padding
        spacing: Theme.paddingSmall
        height: childrenRect.height + 2*padding
        anchors {
            left: parent.left
            leftMargin: showCloseButton ? closeButton.width+Theme.paddingMedium :
                                          contentPadding
            right: attachThumb.left
            rightMargin: contentPadding
        }

        Item { height: 1; width: parent.width } // spacing

        SenderNameLabel {
            id: senderNameLabel
            source: messageData !== null ? getRecipientName(messageData.peerTel, messageData.peerName, false) : ''
            defaultClickAction: false
            anchors { left: parent.left; right: parent.right }
            maximumWidth: parent.width
            horizontalAlignment: root.horizontalAlignment
            highlighted: root.highlighted
            enableBackground: false
        }

        LinkedEmojiLabel {
            id: textLabel
            anchors { left: parent.left; right: parent.right }
            verticalAlignment: Text.AlignTop
            horizontalAlignment: Text.AlignLeft
            plainText: (messageData !== null && messageData.message.trim() !== '') ?
                           messageData.message :
                           ((messageData !== null && messageData.attachments.length > 0) ?
                                //: Placeholder text if quoted message preview contains no text, only attachments
                                //% "Attachment"
                                qsTrId("whisperfish-quoted-message-preview-attachment") :
                                '')
            maximumLineCount: 2
            // height: maximumLineCount*font.pixelSize
            // enableElide: Text.ElideRight -- no elide to enable dynamic height
            font.pixelSize: Theme.fontSizeExtraSmall
            emojiSizeMult: 1.2
            color: root.highlighted ? Theme.secondaryHighlightColor :
                                      Theme.secondaryColor
            linkColor: color
            defaultLinkActions: false
            onLinkActivated: root.clicked(null)
        }
    }

    AttachmentItemBase {
        id: attachThumb
        anchors {
            right: parent.right; rightMargin: 0
            verticalCenter: parent.verticalCenter
        }
        width: attach === null ? 0 : Theme.itemSizeMedium
        height: width
        attach: null
        // XXX: This will work when we expose the quoted message data as QVariantMap instead of a JSON object. Cfr. QtAugmentedMessage::quote(&self).
        // attach: (messageData !== null && messageData.thumbsAttachments.count > 0) ?
        //             messageData.thumbsAttachments.get(0) : null
        enabled: false
        layer.enabled: true
        layer.smooth: true
        layer.effect: RoundedMask {
            // TODO the corners may have to be adapted for different use cases...
            roundedCorners: allCorners
            radius: Theme.paddingSmall
        }
    }
}
