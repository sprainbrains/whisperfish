// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

// TODO the code has to be cleaned up, organized, and documented
//      to ease development of derived types
ListItem {
    id: root
    width: parent.width
    contentHeight: contentContainer.height
    _backgroundColor: "transparent"
    highlighted: down || menuOpen || replyArea.down

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

    // TODO the quoted message should be a notifyable object from a model
    // TODO we need a way to get a valid index from a message id
    //      (we must rely on the message's id instead of its index, as the latter may change)
    // required properties: message, source, outgoing, AND id, index
    property alias quotedMessage: quoteItem.messageData
    readonly property bool hasQuotedMessage: quotedMessage !== null
    property bool quotedMessageShown: hasQuotedMessage

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
    readonly property real minMessageWidth: Math.max(showSender ? senderNameLabel.implicitWidth : 0,
                                                     quotedMessageShown ? quoteItem.implicitWidth : 0,
                                                     showExpand ? maxMessageWidth :
                                                                  statusIcon.width+infoLabel.width+debugLabel.width)

    property bool showExpand: false // this has to be configured by derived items
    readonly property real expandedWidth: width - 2*Theme.horizontalPageMargin // page width
    property bool isExpanded: false // this has to be handled by derived items

    // The parent view can specify a signal to be emitted when
    // the user wants to reply to the delegate's message.
    // Signal signature: \c{replySignal(var index, var modelData)}.
    property var replySignal

    // The parent view can specify a signal to be emitted when
    // the user clicked on the quoted message.
    // Signal signature: \c{quoteClickedSignal(var clickedIndex, var quotedData)}.
    property var quoteClickedSignal

    Component.onCompleted: {
        if (delegateContentWidth <= 0) {
            console.error("No delegateContentWidth specified. List item will not function.")
        }
    }

    Loader {
        id: background
        anchors { fill: contentContainer; margins: contentPadding/3 }
        asynchronous: true
        property real cornerRadius: Theme.paddingLarge
        sourceComponent: Component {
            RoundedRect {
                opacity: (down || replyArea.down) ?
                             (outgoing ? 0.7*Theme.opacityFaint : 1.0*Theme.opacityFaint) :
                             (outgoing ? 0.4*Theme.opacityFaint : 0.8*Theme.opacityFaint)
                color: (down || replyArea.down) ?
                           Theme.highlightBackgroundColor :
                           Theme.secondaryColor
                radius: cornerRadius
                roundedCorners: outgoing ?
                                    bottomLeft | topRight :
                                    bottomRight | topLeft
            }
        }
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
            highlighted: down || root.highlighted
            width: delegateContentWidth
            backgroundGrow: contentPadding/2
            backgroundItem.radius: background.cornerRadius
        }

        Item {
            width: delegateContentWidth
            height: showSender ? senderNameLabel.backgroundGrow+Theme.paddingSmall : 0
        }

        QuotedMessagePreview {
            id: quoteItem
            visible: quotedMessageShown
            width: delegateContentWidth
            maximumWidth: maxMessageWidth
            showCloseButton: false
            showBackground: true
            highlighted: down || root.highlighted
            messageData: null
            backgroundItem.roundedCorners: backgroundItem.bottomLeft |
                                           backgroundItem.bottomRight |
                                           (outgoing ? backgroundItem.topRight :
                                                       backgroundItem.topLeft)
            onClicked: quoteClickedSignal(index, messageData)
        }

        Item {
            width: delegateContentWidth
            height: quoteItem.shown ? Theme.paddingSmall : 0
        }

        Item {
            id: delegateContentItem
            width: delegateContentWidth
            height: childrenRect.height
        }

        Row {
            id: infoRow
            spacing: 0
            layoutDirection: outgoing ? Qt.RightToLeft : Qt.LeftToRight
            width: delegateContentWidth

            HighlightImage {
                id: statusIcon
                visible: outgoing
                width: visible ? Theme.iconSizeSmall : 0
                height: width
                color: infoLabel.color
                source: {
                    if (!modelData) "../../icons/icon-s-queued.png" // cf. below
                    if (modelData.read) "../../icons/icon-s-read.png"
                    else if (modelData.received) "../../icons/icon-s-received.png"
                    else if (modelData.sent) "../../icons/icon-s-sent.png"
                    // TODO actually use 'queued' state in model
                    else if (modelData.queued) "../../icons/icon-s-queued.png"
                    // TODO implement 'failed' state in model
                    // TODO check if SFOS 4 has "image://theme/icon-s-blocked" (3.4 doesn't)
                    else if (modelData.failed) "../../icons/icon-s-failed.png"
                    // TODO If all states are implemented and used, then we should
                    // change the default state to 'failed'. Until then the default
                    // has to be 'queued' to prevent a new message's icon to jump
                    // from 'failed' to 'received'.
                    else "../../icons/icon-s-queued.png"
                }
            }

            Label {
                id: infoLabel
                text: modelData ?
                          (modelData.timestamp ?
                               Format.formatDate(modelData.timestamp, Formatter.TimeValue) :
                               //: Placeholder note if a message doesn't have a timestamp (which must not happen).
                               //% "no time"
                               qsTrId("whisperfish-message-no-timestamp")) :
                          '' // no message to show
                horizontalAlignment: outgoing ? Text.AlignRight : Text.AlignLeft // TODO make configurable
                font.pixelSize: Theme.fontSizeExtraSmall // TODO make configurable
                color: outgoing ?
                           (highlighted ? Theme.secondaryHighlightColor :
                                          Theme.secondaryHighlightColor) :
                           (highlighted ? Theme.secondaryHighlightColor :
                                          Theme.secondaryColor)
            }

            Label {
                id: debugLabel
                visible: SettingsBridge.boolValue("debug_mode")
                width: visible ? implicitWidth : 0
                text: (visible && modelData) ? " [%1] ".arg(modelData.id) : ""
                color: infoLabel.color
                font.pixelSize: Theme.fontSizeExtraSmall
            }

            Row {
                id: showMoreRow
                visible: showExpand
                spacing: Theme.paddingSmall
                layoutDirection: outgoing ? Qt.LeftToRight : Qt.RightToLeft
                width: !visible ? 0 : parent.width - infoLabel.width -
                                      statusIcon.width - debugLabel.width

                Item { width: Theme.paddingSmall; height: 1 }
                Label {
                    font.pixelSize: Theme.fontSizeExtraSmall
                    text: "\u2022 \u2022 \u2022" // three dots
                }
                Label {
                    text: isExpanded ?
                              //: Hint for very long messages, while expanded
                              //% "show less"
                              qsTrId("whisperfish-message-show-less") :
                              //: Hint for very long messages, while not expanded
                              //% "show more"
                              qsTrId("whisperfish-message-show-more")
                    font.pixelSize: Theme.fontSizeExtraSmall
                }
            }
        }
    }

    states: [
        State {
            name: "outgoing"; when: outgoing
            AnchorChanges { target: contentContainer; anchors.right: parent.right }
            AnchorChanges { target: replyArea; anchors.left: parent.left }
            AnchorChanges { target: replyHintIcon; anchors.left: parent.left }
            AnchorChanges { target: replyHintBackground; anchors.left: parent.left }
        },
        State {
            name: "incoming"; when: !outgoing
            AnchorChanges { target: contentContainer; anchors.left: parent.left }
            AnchorChanges { target: replyArea; anchors.right: parent.right }
            AnchorChanges { target: replyHintIcon; anchors.right: parent.right }
            AnchorChanges { target: replyHintBackground; anchors.right: parent.right }
        }
    ]
}
