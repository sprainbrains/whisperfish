import QtQuick 2.6
import Sailfish.Silica 1.0

// This component must be a child of MessageDelegate.
Row {
    id: infoRow
    spacing: 0
    layoutDirection: isOutbound ? Qt.RightToLeft : Qt.LeftToRight
    width: delegateContentWidth

    property real minContentWidth: statusIcon.width+infoLabel.width+debugLabel.width

    HighlightImage {
        id: statusIcon
        visible: isOutbound
        width: visible ? Theme.iconSizeSmall : 0
        height: width
        color: infoLabel.color
        source: {
            if (!hasData) "../../../icons/icon-s-queued.png" // cf. below
            if (modelData.read) "../../../icons/icon-s-read.png"
            else if (modelData.received) "../../../icons/icon-s-received.png"
            else if (modelData.sent) "../../../icons/icon-s-sent.png"
            // TODO actually use 'queued' state in model
            else if (modelData.queued) "../../../icons/icon-s-queued.png"
            // TODO implement 'failed' state in model
            // TODO check if SFOS 4 has "image://theme/icon-s-blocked" (3.4 doesn't)
            else if (modelData.failed) "../../../icons/icon-s-failed.png"
            // TODO If all states are implemented and used, then we should
            // change the default state to 'failed'. Until then the default
            // has to be 'queued' to prevent a new message's icon to jump
            // from 'failed' to 'received'.
            else "../../../icons/icon-s-queued.png"
        }
    }

    Label {
        id: infoLabel
        text: hasData ?
                  (modelData.timestamp ?
                       Format.formatDate(modelData.timestamp, Formatter.TimeValue) :
                       //: Placeholder note if a message doesn't have a timestamp (which must not happen).
                       //% "no time"
                       qsTrId("whisperfish-message-no-timestamp")) :
                  '' // no message to show
        horizontalAlignment: isOutbound ? Text.AlignRight : Text.AlignLeft // TODO make configurable
        font.pixelSize: Theme.fontSizeExtraSmall // TODO make configurable
        color: isOutbound ?
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
        layoutDirection: isOutbound ? Qt.LeftToRight : Qt.RightToLeft
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
