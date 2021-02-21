// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "../../components"

// This component must be a child of MessageDelegate.
Loader {
    id: root
    readonly property var thumbsRe: /^(image|video)\//

    property var thumbsAttachments: _attachments.filter(function(v){ return thumbsRe.test(v.type) })
    property var detailAttachments: _attachments.filter(function(v){ return !thumbsRe.test(v.type) })
    property real thumbsHeight: thumbsAttachments.length > 0 ? Math.min(2*Theme.itemSizeExtraLarge, width) : 0
    property real detailItemHeight: Theme.itemSizeMedium
    property real detailHeight: detailAttachments.length > 0 ? Math.min(maxDetails, detailAttachments.length)*detailItemHeight : 0
    property real spacing: (thumbsAttachments.length > 0 && detailAttachments.length > 0) ? Theme.paddingMedium : 0

    property bool cornersOutbound: false
    property bool cornersQuoted: false

    readonly property int maxDetails: 2
    readonly property int maxThumbs: 5

    // TODO adapt size to screen orientation, i.e. reduce in horizontal mode
    width: 2*Theme.itemSizeExtraLarge
    height: {
        if (!enabled) 0
        else thumbsHeight+detailHeight+spacing
    }

    // TODO Show retry icon
    // TODO Show list of all attachments for >5 images or >2 documents
    // TODO Show image details etc., and actions, on an attached page
    // TODO Stickers: what mime type? custom/variable size?

    sourceComponent: Component {
        Item {
            opacity: highlighted ? Theme.opacityHigh : 1.0
            layer.enabled: true
            layer.effect: RoundedMask {
                id: roundedMask
                roundedCorners: cornersQuoted ? allCorners : (bottomLeft | bottomRight |
                                                              (cornersOutbound ? topRight : topLeft))
                radius: Theme.paddingMedium
            }

            Column {
                width: parent.width
                height: thumbsHeight+detailHeight+root.spacing
                spacing: 0
                Loader {
                    width: parent.width
                    height: thumbsHeight
                    sourceComponent: {
                        if (thumbsAttachments.length === 0) null
                        else if (thumbsAttachments.length === 1) mediaComponent_1
                        else if (thumbsAttachments.length === 2) mediaComponent_2
                        else if (thumbsAttachments.length === 3) mediaComponent_3
                        else if (thumbsAttachments.length === 4) mediaComponent_4
                        else if (thumbsAttachments.length >= 5) mediaComponent_5_plus
                    }
                }

                Item { width: parent.width; height: root.spacing }

                Loader {
                    width: parent.width
                    height: detailHeight
                    sourceComponent: detailComponent
                }
            }
        }
    }

    Component {
        id: mediaComponent_1
        AttachmentThumbnail {
            anchors.fill: parent
            attach: thumbsAttachments[0]
        }
    }

    Component {
        id: mediaComponent_2
        Row {
            AttachmentThumbnail {
                width: parent.width/2; height: parent.height
                attach: thumbsAttachments[0]
            }
            AttachmentThumbnail {
                width: parent.width/2; height: parent.height
                attach: thumbsAttachments[1]
            }
        }
    }

    Component {
        id: mediaComponent_3
        Row {
            AttachmentThumbnail {
                width: parent.width/2; height: parent.height
                attach: thumbsAttachments[0]
            }

            Column {
                width: parent.width/2; height: parent.height
                AttachmentThumbnail {
                    width: parent.width; height: parent.height/2
                    attach: thumbsAttachments[1]
                }
                AttachmentThumbnail {
                    width: parent.width; height: parent.height/2
                    attach: thumbsAttachments[2]
                }
            }
        }
    }

    Component {
        id: mediaComponent_4
        Row {
            Column {
                width: parent.width/2; height: parent.height
                AttachmentThumbnail {
                    width: parent.width; height: parent.height/2
                    attach: thumbsAttachments[0]
                }
                AttachmentThumbnail {
                    width: parent.width; height: parent.height/2
                    attach: thumbsAttachments[1]
                }
            }
            Column {
                width: parent.width/2; height: parent.height
                AttachmentThumbnail {
                    width: parent.width; height: parent.height/2
                    attach: thumbsAttachments[2]
                }
                AttachmentThumbnail {
                    width: parent.width; height: parent.height/2
                    attach: thumbsAttachments[3]
                }
            }
        }
    }

    Component {
        id: mediaComponent_5_plus
        Column {
            Row {
                width: parent.width; height: parent.height/5*3
                AttachmentThumbnail {
                    width: parent.width/2; height: parent.height
                    attach: thumbsAttachments[0]
                }
                AttachmentThumbnail {
                    width: parent.width/2; height: parent.height
                    attach: thumbsAttachments[1]
                }
            }
            Row {
                width: parent.width; height: parent.height/5*2
                AttachmentThumbnail {
                    width: parent.width/3; height: parent.height
                    attach: thumbsAttachments[2]
                }
                AttachmentThumbnail {
                    width: parent.width/3; height: parent.height
                    attach: thumbsAttachments[3]
                }
                AttachmentThumbnail {
                    id: showMoreThumb
                    width: parent.width/3; height: parent.height
                    attach: thumbsAttachments[4]

                    Rectangle {
                        visible: thumbsAttachments.length > maxThumbs
                        anchors.fill: parent
                        color: Theme.highlightDimmerColor
                        opacity: parent.highlighted ? 0.7 : 0.85

                        Label {
                            highlighted: showMoreThumb.highlighted
                            anchors.fill: parent
                            text: "+%1".arg(thumbsAttachments.length-maxThumbs) // translate?
                            fontSizeMode: Text.Fit
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }
                }
            }
        }
    }

    Component {
        id: detailComponent
        Column {
            id: detailColumn
            function componentForMime(mimeType) {
                if (/^audio\//.test(mimeType)) return detail_audioComponent
                else if (/^text\/(x-)?vcard/.test(mimeType)) return detail_contactComponent
                /* else if (mimeType === 'text/x-signal-plain') return null */
                else return detail_fileComponent
            }

            Loader {
                property int currentAttachmentIndex: 0
                width: parent.width
                height: parent.height/Math.min(maxDetails, detailAttachments.length)
                sourceComponent: detailAttachments.length >= 1 ?
                                     parent.componentForMime(detailAttachments[0].type) : null
            }

            Item {
                width: parent.width
                height: showMoreDetail.sourceComponent !== null ? parent.height/maxDetails : 0
                Loader {
                    id: showMoreDetail
                    anchors.fill: parent
                    property int currentAttachmentIndex: 1
                    sourceComponent: detailAttachments.length >= maxDetails ?
                                         detailColumn.componentForMime(detailAttachments[0].type) : null

                }
                Rectangle {
                    visible: detailAttachments.length > maxDetails
                    color: Theme.highlightDimmerColor
                    opacity: highlighted ? 0.7 : 0.85
                    anchors.fill: showMoreDetail
                    property bool highlighted: showMoreDetail.item && showMoreDetail.item.highlighted

                    Label {
                        highlighted: parent.highlighted
                        anchors.fill: parent
                        text: "+%1".arg(detailAttachments.length-maxDetails) // translate?
                        fontSizeMode: Text.Fit
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }

    Component {
        id: detail_contactComponent
        AttachmentItemContact {
            attach: detailAttachments[currentAttachmentIndex]
        }
    }

    Component {
        id: detail_audioComponent
        AttachmentItemAudio {
            attach: detailAttachments[currentAttachmentIndex]
        }
    }

    Component {
        id: detail_fileComponent
        AttachmentItemFile {
            attach: detailAttachments[currentAttachmentIndex]
        }
    }
}
