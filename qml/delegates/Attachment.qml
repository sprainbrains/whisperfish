/*
 * Copyright (C) 2012-2015 Jolla Ltd.
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

import QtQuick 2.2
import QtGraphicalEffects 1.0
import Sailfish.Silica 1.0
import Nemo.Thumbnailer 1.0

Thumbnail {
    id: attachment
    opacity: 1.0
    width: opacity == 1.0 ? size : 0
    height: width
    sourceSize {
        width: size * 2
        height: size * 2
    }

    property bool inbound

    property int radius: 0

    property var messagePart
    property bool showRetryIcon
    property int size: Theme.itemSizeExtraLarge
    property bool highlighted
    property bool isThumbnail: messagePart.mimeType.substr(0, 6) === "image/"
    property bool isVCard: {
        var type = messagePart.mimeType.toLowerCase()
        return type.substr(0, 10) === "text/vcard" || type.substr(0, 12) === "text/x-vcard"
    }

    source: isThumbnail ? messagePart.attachment : ""

    function mimeToIcon(mimeType) {
        var icon = Theme.iconForMimeType(mimeType)
        return icon === "image://theme/icon-m-file-other" ? "image://theme/icon-m-attach" : icon
    }

    Image {
        id: icon
        anchors.fill: parent
        fillMode: Image.Pad
        source: iconSource()

        function iconSource() {
            if (messagePart.mimeType === undefined ||
                messagePart.mimeType.substr(0, 16) === "application/smil" ||
                messagePart.mimeType.substr(0, 10) === "text/plain")
                return ""
            else if (showRetryIcon)
                return "image://theme/icon-m-refresh?" + (message.highlighted ? Theme.highlightColor : Theme.primaryColor)
            else if (isThumbnail && attachment.status !== Thumbnail.Error)
                return ""
            else if (isVCard)
                return "image://theme/icon-m-person" + (highlighted ? "?" + Theme.highlightColor : "")
            else
                return mimeToIcon(messagePart.mimeType) + (highlighted ? "?" + Theme.highlightColor : "")
        }

        Rectangle {
            anchors.fill: parent
            z: -1
            color: Theme.highlightColor
            opacity: 0.1
            visible: true
        }
    }

    layer.enabled: radius != 0
    layer.effect: OpacityMask {
        maskSource: Item {
            width: attachment.width
            height: attachment.height

            // Rectangle that fits the whole thing with all corners rounded
            Rectangle {
                id: roundedRectangle

                anchors.centerIn: parent
                width: attachment.adapt ? attachment.width : Math.min(attachment.width, attachment.height)
                height: attachment.adapt ? attachment.height : width

                radius: attachment.radius
            }

            // Rectangles that recolors the corner without radius, depending on whether it's an inbound or outbound message.
            // The above rectangle cannot specify which corners are (not) rounded, it's all-or-nothing.
            Rectangle {
                // Bottom
                anchors {
                    left: parent.left
                    right: parent.right

                    bottom: parent.bottom
                }

                width: roundedRectangle.width
                height: attachment.radius * 2
            }
            Rectangle {
                anchors {
                    left: !inbound ? parent.left  : undefined
                    right: inbound ? parent.right : undefined

                    top: parent.top
                }

                width: attachment.radius * 2
                height: attachment.radius * 2
            }
        }
    }
}

