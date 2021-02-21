import QtQuick 2.6
import Sailfish.Silica 1.0

// This component must be a child of MessageDelegate.
MouseArea {
    id: replyArea
    enabled: !isEmpty
    property bool down: pressed && containsPress && !menuOpen

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
        rotation: isOutbound ? -90 : 90
        transformOrigin: isOutbound ? Item.TopLeft : Item.TopRight
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
            name: "outbound"; when: isOutbound
            AnchorChanges { target: replyHintIcon; anchors.left: parent.left }
            AnchorChanges { target: replyHintBackground; anchors.left: parent.left }
        },
        State {
            name: "inbound"; when: !isOutbound
            AnchorChanges { target: replyHintIcon; anchors.right: parent.right }
            AnchorChanges { target: replyHintBackground; anchors.right: parent.right }
        }
    ]
}
