// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

ListItem {
    width: parent.width
    contentHeight: contentContainer.height

    // TODO Uncomment this line only for development!
    down: pressed || (enableDebugLayer && (index % 2 == 0))

    property QtObject modelData
    property bool outgoing: modelData.outgoing ? true : false
    property bool hasText: modelData.message ? true : false
    property int index: modelData.index
    property real maxMessageWidth: parent.width -
                                   6*Theme.horizontalPageMargin
    default property alias delegateContents: delegateContentItem.data

    property real contentPadding: 2*Theme.paddingMedium
    property bool enableDebugLayer: true

    RoundedRect {
        id: background
        opacity: outgoing ? Theme.opacityFaint : Theme.opacityHigh
        color: Theme.rgba(Theme.primaryColor, Theme.opacityFaint)
        radius: Theme.paddingLarge
        anchors { fill: contentContainer; margins: contentPadding/3 }
        roundedCorners: outgoing ?
                            bottomLeft | topRight :
                            bottomRight | topLeft
        Behavior on width { SmoothedAnimation { duration: 100 } }
        Behavior on height { SmoothedAnimation { duration: 100 } }
    }

    Column {
        id: contentContainer
        width: childrenRect.width + 2*contentPadding
        height: childrenRect.height + 2*contentPadding
        leftPadding: contentPadding
        rightPadding: contentPadding
        topPadding: contentPadding
        bottomPadding: contentPadding

        anchors {
            top: parent.top
            right: outgoing ? parent.right : undefined//; rightMargin: Theme.paddingMedium
            left: outgoing ? undefined : parent.left//; leftMargin: Theme.paddingMedium
        }

        Item {
            id: delegateContentItem
            width: childrenRect.width
            height: childrenRect.height

            Rectangle {
                visible: enableDebugLayer
                anchors.fill: parent
                color: Theme.highlightDimmerColor
                opacity: 0.4
            }
        }
    }
}
