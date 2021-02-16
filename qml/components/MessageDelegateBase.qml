// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

ListItem {
    id: root
    width: parent.width
    contentHeight: contentContainer.height
    _backgroundColor: "transparent"
    highlighted: down || menuOpen || replyArea.down

    // TODO Uncomment this line only for development!
    // down: pressed || (enableDebugLayer && (index % 2 == 0))
    property bool enableDebugLayer: false

    property QtObject modelData
    property bool outgoing: (modelData !== null && modelData.outgoing) ? true : false
    property bool hasText: (modelData !== null && modelData.message) ? true : false
    readonly property bool hasSource: typeof modelData.source !== 'undefined' &&
                                      modelData.source.trim() !== ''
    property int index: modelData !== null ? modelData.index : -1
    property var contact: outgoing ? null : resolvePeopleModel.personByPhoneNumber(modelData.source)
    property var contactName: contact !== null ? contact.displayLabel : modelData.source
    property ListView listView: ListView.view

    readonly property bool isInGroup: MessageModel.group
    property bool showSender: (isInGroup && !outgoing) || !hasSource

    // All children are placed inside a bubble, positioned
    // left or right for incoming/outgoing messages. The bubble
    // extends slightly over the contents, the list item extends
    // over the bubble.
    property real contentPadding: 2*Theme.paddingMedium
    default property alias delegateContents: delegateContentItem.data

    // Derived types have to set \c delegateContentWidth, which
    // must stay between \c minMessageWidth and \c maxMessageWidth.
    property real delegateContentWidth: -1
    property real maxMessageWidth: parent.width -
                                   6*Theme.horizontalPageMargin
    readonly property real minMessageWidth: Math.max(senderNameLabel.implicitWidth)

    // The parent view can specify a signal to be emitted when
    // the user wants to reply to the delegate's message.
    // Signal signature: \c{replySignal(var index, var modelData)}.
    property var replySignal

    Component.onCompleted: {
        if (delegateContentWidth <= 0) {
            console.error("No delegateContentWidth specified. List item will not function.")
        }
    }

    RoundedRect {
        id: background
        opacity: (down || replyArea.down) ?
                     (outgoing ? 0.7*Theme.opacityFaint : 1.0*Theme.opacityFaint) :
                     (outgoing ? 0.4*Theme.opacityFaint : 0.8*Theme.opacityFaint)
        color: (down || replyArea.down) ?
                   Theme.highlightBackgroundColor :
                   Theme.secondaryColor
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
        enabled: hasText && root.enabled // TODO enable if the message is not empty
        property bool down: pressed && containsPress && !menuOpen

        anchors { top: parent.top; bottom: parent.bottom }
        width: parent.width/2
        onPressAndHold: root.openMenu()
        onClicked: {
            if (replySignal) replySignal(index, modelData)
            else console.error("reply requested but not signal specified")
        }

        HighlightImage {
            id: replyHintIcon
            // alternative icons: outline-chat, bubble-universal, notifications
            source: 'image://theme/icon-m-message-reply'
            asynchronous: true
            anchors.verticalCenter: parent.verticalCenter
            opacity: replyHintBackground.opacity
            enabled: false
            color: Theme.secondaryColor
            anchors.margins: Theme.horizontalPageMargin
        }

        Rectangle {
            id: replyHintBackground
            width: parent.height
            height: Math.max(parent.width, root.width-delegateContentWidth)
            rotation: outgoing ? -90 : 90
            transformOrigin: outgoing ? Item.TopLeft : Item.TopRight
            y: parent.height
            opacity: parent.down ? 1.0 : 0.0
            gradient: Gradient {
                GradientStop { position: 0.2; color: Theme.rgba(Theme.highlightBackgroundColor,
                                                                Theme.highlightBackgroundOpacity) }
                GradientStop { position: 1.0; color: "transparent" }
            }
            Behavior on opacity { FadeAnimation { duration: 50 } }
        }

        states: [
            State {
                name: "outgoing"; when: outgoing
                AnchorChanges { target: replyArea; anchors.left: parent.left }
                AnchorChanges { target: replyHintIcon; anchors.left: parent.left }
                AnchorChanges { target: replyHintBackground; anchors.left: parent.left }
            },
            State {
                name: "incoming"; when: !outgoing
                AnchorChanges { target: replyArea; anchors.right: parent.right }
                AnchorChanges { target: replyHintIcon; anchors.right: parent.right }
                AnchorChanges { target: replyHintBackground; anchors.right: parent.right }
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

        // IMPORTANT Never use 'parent.width' in this content container!
        //           This breaks width calculations here and in derived items.
        //           Always use delegateContentWidth instead.

        SenderNameLabel {
            id: senderNameLabel
            visible: showSender
            text: hasSource ?
                      contactName :
                      //: Label shown if a message doesn't have a sender.
                      //% "no sender"
                      qsTrId("whisperfish-sender-label-empty")
            source: (outgoing || !hasSource) ? '' : modelData.source
            outbound: root.outgoing
            maximumWidth: maxMessageWidth
            width: delegateContentWidth
            backgroundGrow: contentPadding/2
            backgroundItem.radius: background.radius
        }
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
