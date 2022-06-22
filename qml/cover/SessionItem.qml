import QtQuick 2.0
import Sailfish.Silica 1.0

Column {
    id: root

    property var session
    property var contact: (session.isGroup || !mainWindow.contactsReady) ? null : resolvePeopleModel.personByPhoneNumber(session.source, true)
    property string message: (session.hasAttachment && session.message === ''
            // TODO we could show an icon in front
            //: Session contains an attachment label
            //% "Attachment"
            ? qsTrId("whisperfish-session-has-attachment")
            : session.message
        )

    Item {
        width: textLabel.width
        height: textLabel.height
        Label {
            id: textLabel

            font.pixelSize: Theme.fontSizeSmall
            verticalAlignment: Text.AlignTop

            width: root.width

            maximumLineCount: 2
            wrapMode: Text.Wrap

            color: session.read ? Theme.primaryColor : Theme.highlightColor

            text: session.message
        }

        OpacityRampEffect {
            offset: 0.5
            sourceItem: textLabel
        }
    }

    Row {
        id: recipientRow
        spacing: Theme.paddingSmall

        width: root.width - Theme.paddingLarge

        Item {
            width: recipientLabel.width
            height: recipientLabel.height

            Label {
                id: recipientLabel

                font.pixelSize: Theme.fontSizeExtraSmall
                verticalAlignment: Text.AlignTop
                width: typingIcon.visible ? (recipientRow.width - typingIcon.width - Theme.paddingSmall) : recipientRow.width

                maximumLineCount: 1
                truncationMode: TruncationMode.Fade

                color: session.read ? Theme.highlightColor : Theme.secondaryHighlightColor

                text: SetupWorker.phoneNumber === model.source ?
                          //: Name of the conversation with one's own number
                          //% "Note to self"
                          qsTrId("whisperfish-session-note-to-self") :
                          session.isGroup ? session.groupName : ( contact == null ? session.source : contact.displayLabel )
            }

            OpacityRampEffect {
                sourceItem: recipientLabel
                enabled: typingIcon.visible || recipientLabel.implicitWidth > recipientLabel.width
                offset: 0.5
            }
        }

        Item {
            width: typingIcon.width
            height: typingIcon.height
            Image {
                id: typingIcon

                source: "image://theme/icon-m-bubble-universal"

                visible: session.isTyping

                fillMode: Image.PreserveAspectFit
                height: recipientLabel.height

                Behavior on opacity {
                    FadeAnimation {}
                }
            }

            Timer {
                running: typingIcon.visible
                repeat: true
                interval: 100

                property bool direction: true;

                onTriggered: {
                    // All this because I have no idea whether you can set a speed on the animation
                    if (direction) {
                        typingIcon.opacity += 0.2;
                        if (typingIcon.opacity >= 1) {
                            direction = !direction;
                        }
                    } else {
                        typingIcon.opacity -= 0.2;
                        if (typingIcon.opacity <= .5) {
                            direction = !direction;
                        }
                    }
                }
            }
        }
    }
}
