import QtQuick 2.2
import Sailfish.Silica 1.0

// Note: This is the Whisperfish Captcha application cover page.
// Do not use directly within Whisperfish.

CoverBackground {
    Label {
        anchors {
            top: parent.top
            topMargin: Theme.paddingLarge
            horizontalCenter: parent.horizontalCenter
        }
        horizontalAlignment: Text.AlignHCenter
        // Not seen normally, can be left untranslated
        text: "Whisperfish\captcha"
    }

    Image {
        x: Theme.paddingLarge
        horizontalAlignment: Text.AlignHCenter
        width: parent.width * 0.3
        height: width
        source: "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
        anchors {
            bottom: parent.bottom
            bottomMargin: Theme.itemSizeHuge
            horizontalCenter: parent.horizontalCenter
        }
    }
}
