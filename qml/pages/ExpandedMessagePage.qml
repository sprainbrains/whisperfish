import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"
import "../delegates"

Page {
    id: root
    property QtObject modelData
    property bool outgoing
    property string messageText: modelData.message.trim()

    property string _originName: _contact !== null ? _contact.displayLabel : ''
    property var _contact: (modelData !== null && mainWindow.contactsReady) ?
                               resolvePeopleModel.personByPhoneNumber(modelData.source) : null


    SilicaFlickable {
        id: flick
        anchors.fill: parent
        contentHeight: column.height

        VerticalScrollDecorator { flickable: flick }

        Column {
            id: column
            width: parent.width
            spacing: Theme.paddingMedium

            PageHeader {
                //: TODO
                //% "Full message"
                title: qsTrId("whisperfish-expanded-message-page-header")
                description: (outgoing ?
                                 //: TODO
                                 //% "to %1 at %2 o'clock"
                                 qsTrId("whisperfish-expanded-message-info-outgoing") :
                                 //: TODO
                                 //% "from %1 at %2 o'clock"
                                 qsTrId("whisperfish-expanded-message-info-incoming")).
                                    arg(_originName).arg(modelData.timestamp ?
                                        Format.formatDate(modelData.timestamp, Formatter.TimeValue) :
                                        //: Placeholder note if a message doesn't have a timestamp (which must not happen).
                                        //% "no time"
                                        qsTrId("whisperfish-message-no-timestamp"))
            }

            MessageDelegateBase {
                id: delegate
                modelData: root.modelData
                enabled: false
                delegateContentWidth: messageLabel.width

                LinkedEmojiLabel {
                    id: messageLabel
                    wrapMode: Text.Wrap
                    width: root.width - 4*Theme.horizontalPageMargin
                    horizontalAlignment: outgoing ? Text.AlignRight : Text.AlignLeft // TODO make configurable
                    color: highlighted ? Theme.highlightColor :
                                         (outgoing ? Theme.highlightColor :
                                                     Theme.primaryColor)
                    font.pixelSize: Theme.fontSizeSmall // TODO make configurable
                    plainText: messageText
                }
            }

            Item { width: parent.width; height: Theme.horizontalPageMargin }
        }
    }
}
