import QtQuick 2.2
import Sailfish.Silica 1.0

Dialog {
    id: page
    objectName: "selectContact"
    canAccept: false
    allowedOrientations: Orientation.All

    property string selectedContact: ""
    property var contactList
    signal selected(string name, string tel)

    Component.onCompleted: function() {
        console.log("contactList: "+contactList)
    }

    SilicaFlickable {
        id: sc
        focus: true
        contentHeight: scc.y + scc.height
        anchors.fill: parent

        Column {
            id: scc
            width: parent.width
            spacing: Theme.paddingLarge

            DialogHeader {
                id: title
                //: Title for select contact page
                //% "Select contact"
                title: qsTrId("whisperfish-select-contact")
                acceptText: ""
            }

            AlphaMenu {
                id: alphaMenu
                dataSource: contactList
                listDelegate:  BackgroundItem {
                    id: contactItem
                    width: parent.width
                    onClicked: {
                        highlighted = !highlighted  
                        page.selected(name, tel)
                        page.close()
                    }
                    Row {
                        spacing: 20

                        Column {
                            Label {
                                text: name
                                font.pixelSize: Theme.fontSizeMedium
                                color: Theme.primaryColor
                            }
                            Label {
                                text: tel
                                font.pixelSize: Theme.fontSizeExtraSmall
                                color: Theme.secondaryColor
                            }
                        }
                    }
                }
            }
        }

        VerticalScrollDecorator {}
    }
}
