import QtQuick 2.2
import Sailfish.Silica 1.0
import be.rubdos.whisperfish 1.0

CoverBackground {

    Label {
        id: placeholderLabel
        visible: sessionList.count === 0
        text: "Whisperfish"
        anchors.centerIn: parent

        width: Math.min(parent.width, parent.height) * 0.8
        height: width
        font.pixelSize: Theme.fontSizeHuge
        fontSizeMode: Text.Fit
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }

    Label {
        id: unreadCount
        text: SessionModel.unread
        anchors {
            top: parent.top
            left: parent.left
            topMargin: Theme.paddingMedium
            leftMargin: Theme.paddingLarge
        }
        font.pixelSize: Theme.fontSizeHuge

        visible: opacity > 0.0
        opacity: SessionModel.unread > 0 ? 1.0 : 0.0
        Behavior on opacity { NumberAnimation {} }
    }

    Label {
        id: unreadLabel

        //: Unread messages count cover label. Code requires exact line break tag "<br/>".
        //% "Unread<br/>message(s)"
        text: qsTrId("whisperfish-cover-unread-label", SessionModel.unread).replace("<br/>", "\n")
        font.pixelSize: Theme.fontSizeExtraSmall
        maximumLineCount: 2
        wrapMode: Text.Wrap
        fontSizeMode: Text.HorizontalFit
        lineHeight: 0.8
        height: implicitHeight/0.8
        verticalAlignment: Text.AlignVCenter

        visible: opacity > 0.0
        opacity: SessionModel.unread > 0 ? 1.0 : 0.0
        Behavior on opacity { NumberAnimation {} }

        anchors {
            right: parent.right
            rightMargin: Theme.paddingMedium
            left: unreadCount.right
            leftMargin: Theme.paddingMedium
            baseline: unreadCount.baseline
            baselineOffset: lineCount > 1 ? -implicitHeight/2 : -(height-implicitHeight)/2
        }
    }

    OpacityRampEffect {
        offset: 0.75
        slope: 4
        sourceItem: unreadLabel
        enabled: unreadLabel.contentWidth > unreadLabel.width
    }

    SilicaListView {
        id: sessionList
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
            topMargin: Theme.paddingMedium + (SessionModel.unread > 0 ? unreadCount.height : Theme.paddingMedium)
            leftMargin: Theme.paddingLarge
            bottom: coverActionArea.top
            Behavior on topMargin { NumberAnimation {} }
        }

        // XXX Maybe we can use a delegate model to resort without pinning.
        //     Pins don't make a lot of sense in this context
        model: SessionModel
        spacing: Theme.paddingSmall

        delegate: Item {
            width: sessionList.width
            height: messageLabel.height + recipientLabel.height

            Label {
                id: messageLabel
                anchors {
                    top: parent.top
                    left: parent.left
                    right: parent.right
                }
                width: parent.width
                font.pixelSize: Theme.fontSizeExtraSmall
                color: Theme.primaryColor
                truncationMode: TruncationMode.Fade
                text: model.message
            }

            Label {
                id: recipientLabel
                anchors {
                    top: messageLabel.bottom
                    left: parent.left
                    right: parent.right
                }
                font.pixelSize: Theme.fontSizeTiny
                color: Theme.highlightColor
                truncationMode: TruncationMode.Fade
                text: model.isGroup ? model.groupName : getRecipientName(model.source, model.recipientName, false)
            }
        }
    }
    OpacityRampEffect {
        offset: 0.75
        slope: 4
        sourceItem: sessionList
        direction: OpacityRamp.TopToBottom
    }

    Image {
        source: "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
        anchors.centerIn: parent
        width: Math.max(parent.width, parent.height)
        height: width
        fillMode: Image.PreserveAspectFit
        opacity: 0.1
    }

    CoverActionList {
        id: coverAction
        // enabled: !placeholderLabel.visible
        // XXX https://gitlab.com/whisperfish/whisperfish/-/issues/481
        enabled: false
        CoverAction {
            iconSource: {
                if (ClientWorker.connected) {
                    return "/usr/share/harbour-whisperfish/icons/172x172/connected.png"
                } else if (!ClientWorker.connected) {
                    return "/usr/share/harbour-whisperfish/icons/172x172/disconnected.png"
                } else {
                    return "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
                }
            }
            onTriggered: {
                if(!SetupWorker.locked) {
                    mainWindow.activate()
                    mainWindow.newMessage(PageStackAction.Immediate)
                }
            }
        }
    }
}
