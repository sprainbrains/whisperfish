/*
 * Copyright (C) 2012-2015 Jolla Ltd.
 *                    2020 Nicolas Werner
 *                    2020 Ruben De Smet
 *
 * The code in this file is distributed under multiple licenses, and as such,
 * may be used under any one of the following licenses:
 *
 *   - GNU General Public License as published by the Free Software Foundation;
 *     either version 2 of the License (see LICENSE.GPLv2 in the root directory
 *     for full terms), or (at your option) any later version.
 *   - GNU Lesser General Public License as published by the Free Software
 *     Foundation; either version 2.1 of the License (see LICENSE.LGPLv21 in the
 *     root directory for full terms), or (at your option) any later version.
 *   - Alternatively, if you have a commercial license agreement with Jolla Ltd,
 *     you may use the code under the terms of that license instead.
 *
 * You can visit <https://sailfishos.org/legal/> for more information
 */

import QtQuick 2.6
import QtQuick.Layouts 1.0
import Sailfish.Silica 1.0
import Sailfish.Silica 1.0
import Sailfish.Silica.private 1.0
import Nemo.Thumbnailer 1.0

// BubbleBackground inspired on Konheko, GPLv3+
// https://nheko.im/nheko-reborn/konheko/-/blob/master/qml/pages/ChatPage.qml#L101
ListItem {
    id: messageItem
    contentHeight: content.height + 2 * Theme.paddingMedium
    menu: messageContextMenu
    width: conversation.width

    property QtObject modelData
    property bool inbound: modelData.outgoing ? false : true
    property bool hasText

    BubbleBackground {
        id: bubble

        property int maximumMessageWidth: parent.width - 2 * Theme.paddingLarge
        property int index: modelData.index

        color: Theme.rgba(Theme.primaryColor, Theme.opacityFaint)
        opacity: modelData.outgoing ? Theme.opacityFaint : Theme.opacityHigh
        width: content.width
        height: content.height

        radius: Theme.paddingLarge
        roundedCorners: {
            if (modelData.outgoing) {
                BubbleBackground.BottomLeft | BubbleBackground.TopRight
            } else {
                BubbleBackground.BottomRight | BubbleBackground.TopLeft
            }
        }

        anchors {
            topMargin: Theme.paddingSmall
            bottomMargin: Theme.paddingSmall
            leftMargin: Theme.paddingMedium
            rightMargin: Theme.paddingMedium
            right: modelData.outgoing ? parent.right : undefined
            left: !modelData.outgoing ? parent.left : undefined
            top: parent.top
        }

        Behavior on width {
            NumberAnimation {
                duration: 100
                easing.type: Easing.InOutQuad
            }
        }

        Behavior on height {
            NumberAnimation {
                id: contentHeightAnimation

                duration: 100
                easing.type: Easing.InOutQuad
            }
        }
    }

    Row {
        id: content
        width: Math.min(implicitWidth, bubble.maximumMessageWidth)

        layoutDirection: inbound ? Qt.LeftToRight : Qt.RightToLeft

        anchors {
            margins: 0
            right: modelData.outgoing ? bubble.right : undefined
            left: !modelData.outgoing ? bubble.left : undefined
            top: bubble.top
        }

        Column {
            id: attachmentBox

            Repeater {
                id: attachmentLoader
                model: modelData.hasAttachment ? 1 : 0
                property QtObject attachmentItem: modelData

                Attachment {
                    messagePart: attachmentLoader.attachmentItem
                    showRetryIcon: false
                    highlighted: messageItem.highlighted

                    radius: Theme.paddingLarge

                    inbound: messageItem.inbound
                }
            }
        }

        Column {
            id: contentColumn

            height: Math.max(implicitHeight, attachmentBox.height)

            bottomPadding: Theme.paddingSmall
            topPadding: Theme.paddingSmall

            leftPadding:   inbound ? Theme.paddingMedium : Theme.paddingLarge
            rightPadding: !inbound ? Theme.paddingMedium : Theme.paddingLarge

            LinkedLabel {
                id: messageText
                width: {
                    Math.max(
                        Math.min(implicitWidth, bubble.maximumMessageWidth - attachmentBox.width - 2 * Theme.paddingMedium),
                        timestampLabel.implicitWidth + timestampLabel.anchors.rightMargin + timestampLabel.anchors.leftMargin
                    )
                }
                wrapMode: Text.Wrap

                plainText: {
                    if (modelData.message != "") {
                        hasText = true
                        return modelData.message
                    } else {
                        hasText = false
                        return ""
                    }
                }

                color: (messageItem.highlighted || !inbound) ? Theme.highlightColor : Theme.primaryColor
                font.pixelSize: inbound ? Theme.fontSizeMedium : Theme.fontSizeSmall
                horizontalAlignment: inbound ? Qt.AlignLeft : Qt.AlignRight
                verticalAlignment: Qt.AlignBottom
            }

            // Padding to get the timestampLabel tied to the bottom.
            Item {
                width: 1
                height: attachmentBox.height - (timestampLabel.height + messageText.height + contentColumn.bottomPadding + contentColumn.topPadding)
                visible: height > 0
            }

            Label {
                id: timestampLabel
                anchors {
                    topMargin: Theme.paddingMedium
                    right: inbound ? undefined : contentColumn.right
                    left: !inbound ? undefined : contentColumn.left
                    rightMargin: parent.rightPadding
                    leftMargin: parent.leftPadding
                }

                function msgDate() {
                    var dt = new Date(modelData.timestamp)
                    var md = Format.formatDate(dt, Formatter.Timepoint)
                    return md
                }

                color: messageText.color
                opacity: 0.6
                font.pixelSize: Theme.fontSizeExtraSmall
                horizontalAlignment: messageText.horizontalAlignment

                text: {
                   var re = msgDate()
                   if (modelData.received) {
                       re += "  ✓✓"
                   } else if (modelData.sent) {
                       re += "  ✓"
                   } else if (modelData.queued) {
                       re += "  x"
                   }
                   if(inbound && MessageModel.group) {
                       re += " | " + ContactModel.name(modelData.source)
                   }
                   re += "\n[" + modelData.id + "]"
                   return re
                }
            }
        }
    }

    onClicked: {
        if (modelData.hasAttachment && attachmentBox.height > 0) {
            if(modelData.mimeType == "video/mp4") {
                pageStack.push(Qt.resolvedUrl("../pages/VideoAttachment.qml"), { 'message': modelData })
            } else {
                pageStack.push(Qt.resolvedUrl("../pages/AttachmentPage.qml"), { 'message': modelData })

            }
        }
    }
}

