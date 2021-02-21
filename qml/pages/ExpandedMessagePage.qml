import QtQuick 2.2
import Sailfish.Silica 1.0
import "../components"
import "../delegates"

Page {
    id: root
    property QtObject modelData
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
                description: (delegate.isOutbound ?
                                  //: TODO
                                  //% "to %1"
                                  qsTrId("whisperfish-expanded-message-info-outbound") :
                                  //: TODO
                                  //% "from %1"
                                  qsTrId("whisperfish-expanded-message-info-inbound")).
                              arg(_originName)
            }

            MessageDelegate {
                id: delegate
                modelData: root.modelData
                enabled: false
                delegateContentWidth: root.width - 4*Theme.horizontalPageMargin
                isExpanded: true
                showExpand: false
                extraPageTreshold: _message.length
            }

            Item { width: parent.width; height: Theme.horizontalPageMargin }
        }
    }
}
