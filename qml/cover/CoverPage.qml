import QtQuick 2.2
import Sailfish.Silica 1.0

CoverBackground {
    Image {
        x: Theme.paddingLarge
        horizontalAlignment: Text.AlignHCenter
        width: parent.width * 0.3
        height: width
        source: {
            if(SessionModel.unread > 0) {
                return "/usr/share/harbour-whisperfish/icons/172x172/gold.png"
            } else if(ClientWorker.connected) {
                return "/usr/share/harbour-whisperfish/icons/172x172/connected.png"
            } else if(!ClientWorker.connected) {
                return "/usr/share/harbour-whisperfish/icons/172x172/disconnected.png"
            } else {
                return "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
            }
        }
        anchors {
            bottom: parent.bottom
            bottomMargin: Theme.itemSizeHuge
            horizontalCenter: parent.horizontalCenter
        }
    }

    CoverActionList {
        CoverAction {
            iconSource: "image://theme/icon-cover-message"
            onTriggered: {
                if(!SetupWorker.locked) {
                    mainWindow.activate()
                    mainWindow.newMessage(PageStackAction.Immediate)
                }
            }
        }
    }

    Column {
        x: Theme.paddingLarge
        spacing: Theme.paddingSmall
        width: parent.width - 2*Theme.paddingLarge
        UnreadLabel {
            id: unreadLabel
        }
    }
}
