// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

ListItem {
    width: parent.width
    contentHeight: contentContainer.height

    // TODO Uncomment this line only for development!
    // down: pressed || (enableDebugLayer && (index % 2 == 0))
    property bool enableDebugLayer: false

    property QtObject modelData
    property bool outgoing: modelData.outgoing ? true : false
    property bool hasText: modelData.message ? true : false
    property int index: modelData.index
    property ListView listView: ListView.view

    // All children are placed inside a bubble, positioned
    // left or right for incoming/outgoing messages. The bubble
    // extends slightly over the contents, the list item extends
    // over the bubble.
    property real contentPadding: 2*Theme.paddingMedium
    default property alias delegateContents: delegateContentItem.data

    // Derived types have to set \c delegateContentWidth, which
    // should (read: must) stay between \c minMessageWidth and
    // \c maxMessageWidth.
    property real delegateContentWidth: -1
    property real maxMessageWidth: parent.width -
                                   6*Theme.horizontalPageMargin

    Component.onCompleted: {
        if (delegateContentWidth <= 0) {
            console.error("No delegateContentWidth specified. List item will not function.")
        }
    }

    RoundedRect {
        id: background
        opacity: outgoing ? Theme.opacityFaint : Theme.opacityHigh
        color: Theme.rgba(Theme.primaryColor, Theme.opacityFaint)
        radius: Theme.paddingLarge
        anchors { fill: contentContainer; margins: contentPadding/3 }
        roundedCorners: outgoing ?
                            bottomLeft | topRight :
                            bottomRight | topLeft
    }

    Rectangle {
        visible: enableDebugLayer
        anchors.fill: contentContainer
        color: Theme.highlightDimmerColor
        opacity: 0.4
    }

    MouseArea {
        id: replyArea
        anchors { top: parent.top; bottom: parent.bottom }
        width: parent.width/2
        onPressAndHold: parent.openMenu()
        onClicked: {
            console.log("replying is not implemented yet")
        }

        HighlightImage {
            id: button
            // alternative icons: icon-m-outline-chat icon-m-bubble-universal
            source: 'image://theme/icon-m-message-reply'
            asynchronous: true
            anchors.verticalCenter: parent.verticalCenter
            opacity: parent.pressed ? 1.0 : 0.0
            Behavior on opacity { FadeAnimator { duration: 100 } }
            enabled: false
            color: Theme.secondaryColor
            anchors.margins: Theme.horizontalPageMargin
        }

        Rectangle {
            id: bg
            width: parent.height
            height: parent.width
            rotation: outgoing ? -90 : 90
            transformOrigin: outgoing ? Item.TopLeft : Item.TopRight
            y: parent.height
            opacity: parent.pressed ? 1.0 : 0.0
            gradient: Gradient {
                GradientStop { position: 0.0; color: Theme.rgba(Theme.highlightBackgroundColor,
                                                                Theme.highlightBackgroundOpacity) }
                GradientStop { position: 1.0; color: "transparent" }
            }
            Behavior on opacity { FadeAnimator { duration: 200 } }
        }

        states: [
            State {
                name: "outgoing"; when: outgoing
                AnchorChanges { target: replyArea; anchors.left: parent.left }
                AnchorChanges { target: button; anchors.left: parent.left }
                AnchorChanges { target: bg; anchors.left: parent.left }
            },
            State {
                name: "incoming"; when: !outgoing
                AnchorChanges { target: replyArea; anchors.right: parent.right }
                AnchorChanges { target: button; anchors.right: parent.right }
                AnchorChanges { target: bg; anchors.right: parent.right }
            }
        ]
    }

    Column {
        id: contentContainer
        padding: contentPadding
        anchors {
            // The text should be aligned with other page elements
            // by having the default side margins. The bubble should
            // extend a little bit over the margins.
            top: parent.top
            rightMargin: Theme.horizontalPageMargin - contentPadding
            leftMargin: Theme.horizontalPageMargin - contentPadding
        }

        Item {
            id: delegateContentItem
            width: delegateContentWidth
            height: childrenRect.height
        }

        states: [
            State {
                name: "outgoing"; when: outgoing
                AnchorChanges { target: contentContainer; anchors.right: parent.right }
            },
            State {
                name: "incoming"; when: !outgoing
                AnchorChanges { target: contentContainer; anchors.left: parent.left }
            }
        ]
    }
}
