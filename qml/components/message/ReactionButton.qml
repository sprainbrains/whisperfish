import QtQuick 2.6
import Sailfish.Silica 1.0
import "../"

LinkedEmojiLabel {
    property var menuItem
    property bool remove
    height: reactMenuItem.height
    width: reactMenuItem.height
    horizontalAlignment: Text.AlignHCenter
    verticalAlignment: Text.AlignVCenter
    MouseArea {
        anchors.fill: parent
        onClicked: {
            reactInline(
                menuItem.parent,
                parent.plainText,
                parent.remove
            )
            menuItem.close()
        }
    }
}
