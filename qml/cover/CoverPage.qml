import QtQuick 2.2
import Sailfish.Silica 1.0
import be.rubdos.whisperfish 1.0

CoverBackground {
    property bool rightToLeft: Qt.application.layoutDirection === Qt.RightToLeft
    Connections {
        target: SetupWorker
        onSetupComplete: {
            sessions.reinit()
        }
    }

    Sessions {
        id: sessions
        app: AppState
    }

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
        text: sessions.unread
        anchors {
            top: parent.top
            left: rightToLeft ? undefined : parent.left
            right: rightToLeft ? parent.right : undefined
            topMargin: Theme.paddingMedium
            leftMargin: rightToLeft ? undefined : Theme.paddingLarge
            rightMargin: rightToLeft ? Theme.paddingLarge : undefined
        }
        font.pixelSize: Theme.fontSizeHuge

        visible: opacity > 0.0
        opacity: sessions.unread > 0 ? 1.0 : 0.0
        Behavior on opacity { NumberAnimation {} }
    }

    Label {
        id: unreadLabel

        //: Unread messages count cover label. Code requires exact line break tag "<br/>".
        //% "Unread<br/>message(s)"
        text: qsTrId("whisperfish-cover-unread-label", sessions.unread).replace("<br/>", "\n")
        font.pixelSize: Theme.fontSizeExtraSmall
        maximumLineCount: 2
        wrapMode: Text.Wrap
        fontSizeMode: Text.HorizontalFit
        lineHeight: 0.8
        height: implicitHeight/0.8
        verticalAlignment: Text.AlignVCenter

        visible: opacity > 0.0
        opacity: sessions.unread > 0 ? 1.0 : 0.0
        Behavior on opacity { NumberAnimation {} }

        anchors {
            right: rightToLeft ? unreadCount.left : parent.right
            rightMargin: Theme.paddingMedium
            left: rightToLeft ? parent.left : unreadCount.right
            leftMargin: Theme.paddingMedium
            baseline: unreadCount.baseline
            baselineOffset: lineCount > 1 ? -implicitHeight/2 : -(height-implicitHeight)/2
        }
    }

    OpacityRampEffect {
        offset: 0.9
        slope: 10
        sourceItem: unreadLabel
        enabled: unreadLabel.contentWidth > unreadLabel.width
        direction: rightToLeft ? OpacityRamp.RightToLeft : OpacityRamp.LeftToRight
    }

    SilicaListView {
        id: sessionList
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
            topMargin: Theme.paddingMedium + (sessions.unread > 0 ? unreadCount.height : Theme.paddingMedium)
            leftMargin: Theme.paddingLarge
            rightMargin: Theme.paddingLarge
            bottom: coverActionArea.top
            Behavior on topMargin { NumberAnimation {} }
        }

        // XXX Maybe we can use a delegate model to resort without pinning.
        //     Pins don't make a lot of sense in this context
        model: sessions.sessions
        spacing: Theme.paddingSmall

        delegate: Item {
            width: sessionList.width
            height: model.isArchived ? 0 : messageLabel.height + recipientLabel.height
            enabled: !model.isArchived
            visible: !model.isArchived

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
                text: (model.hasAttachment
                    ? ("📎 " + (model.message === ''
                        // SessionDelegate defines this
                        ? qsTrId("whisperfish-session-has-attachment") : '')
                    ) : ''
                ) + (model.message !== undefined ? model.message : '')
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
                text: model.isGroup ? model.groupName : getRecipientName(model.recipientE164, model.recipientName, false)
            }
        }
    }
    OpacityRampEffect {
        offset: 0.8
        slope: 5
        sourceItem: sessionList
        direction: OpacityRamp.TopToBottom
    }

    Image {
        source: "/usr/share/be.rubdos.harbour.whisperfish/icons/172x172/cover-background.png"
        anchors.centerIn: parent
        width: Math.max(parent.width, parent.height)
        height: width
        fillMode: Image.PreserveAspectFit
        opacity: 0.2
    }

    CoverActionList {
        id: coverAction
        enabled: !placeholderLabel.visible
        CoverAction {
            iconSource: {
                if (ClientWorker.connected) {
                    return "/usr/share/be.rubdos.harbour.whisperfish/icons/172x172/connected.png"
                } else if (!ClientWorker.connected) {
                    return "/usr/share/be.rubdos.harbour.whisperfish/icons/172x172/disconnected.png"
                } else {
                    return "/usr/share/icons/hicolor/172x172/apps/be.rubdos.harbour.whisperfish.png"
                }
            }
            onTriggered: {
                if(!SetupWorker.locked) {
                    mainWindow.activate()
                    // XXX https://gitlab.com/whisperfish/whisperfish/-/issues/481
                    // mainWindow.newMessage(PageStackAction.Immediate)
                }
            }
        }
    }
}
