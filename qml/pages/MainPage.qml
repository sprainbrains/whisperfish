import QtQuick 2.2
import Sailfish.Silica 1.0
import "../delegates"

Page {
    id: main
    objectName: mainPageName

    SilicaListView {
        id: sessionView
        model: SessionModel
        anchors.fill: parent
        spacing: Theme.paddingMedium
        footer: Item { width: parent.width; height: Theme.paddingMedium }

        PullDownMenu {
            MenuItem {
                //: About whisperfish menu item
                //% "About Whisperfish"
                text: qsTrId("whisperfish-about-menu")
                onClicked: pageStack.push(Qt.resolvedUrl("About.qml"))
            }
            MenuItem {
                //: Whisperfish settings menu item
                //% "Settings"
                text: qsTrId("whisperfish-settings-menu")
                enabled: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("Settings.qml"))
            }
            MenuItem {
                //: Whisperfish new group menu item
                //% "New Group"
                text: qsTrId("whisperfish-new-group-menu")
                enabled: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("NewGroup.qml"))
            }
            MenuItem {
                //: Whisperfish new message menu item
                //% "New Message"
                text: qsTrId("whisperfish-new-message-menu")
                enabled: !SetupWorker.locked
                onClicked: pageStack.push(Qt.resolvedUrl("NewMessage.qml"))
            }
        }

        VerticalScrollDecorator {}

        ViewPlaceholder {
            enabled: sessionView.count == 0
            // always show app name as placeholder, as the page
            // has not title which might be confusing
            text: "Whisperfish"
            hintText: {
                if (!SetupWorker.registered) {
                    //: Whisperfish registration required message
                    //% "Registration required"
                    qsTrId("whisperfish-registration-required-message")
                } else if (SetupWorker.locked) {
                    //: Whisperfish locked message
                    //% "Locked"
                    qsTrId("whisperfish-locked-message")
                } else {
                    //: No messages found, hint on what to do
                    //% "Pull down to start a new conversation."
                    qsTrId("whisperfish-no-messages-hint-text")
                }
            }
        }

        section {
            property: 'section'
            delegate: SectionHeader {
                height: Theme.itemSizeExtraSmall
                text: {
                    switch(section) {
                    case "today":
                        //: Session section label for today
                        //% "Today"
                        qsTrId("whisperfish-session-section-today")
                        break;
                    case "yesterday":
                        //: Session section label for yesterday
                        //% "Yesterday"
                        qsTrId("whisperfish-session-section-yesterday")
                        break;
                    case "older":
                        //: Session section label for older
                        //% "Older"
                        qsTrId("whisperfish-session-section-older")
                        break;
                    default:
                        // two days to one week ago
                        Qt.locale().standaloneDayName(parseInt(section), Locale.LongFormat)
                    }
                }
            }
        }

        delegate: SessionDelegate {
            onClicked: {
                console.log("Activating session: "+model.id)
                mainWindow.clearNotifications(model.id)
                pageStack.push(Qt.resolvedUrl("Conversation.qml"));
                if (model.unread) {
                    SessionModel.markRead(model.id)
                }
                MessageModel.load(model.id, ContactModel.name(model.source))
            }
        }
    }

    // >>>> TODO REMOVE WITH https://gitlab.com/rubdos/whisperfish/-/merge_requests/91
    Connections {
        target: Prompt
        onPromptPhoneNumber: {
            phoneNumberTimer.start()
        }
        onPromptVerificationCode: {
            verifyTimer.start()
        }
        onPromptPassword: {
            passwordTimer.start()
        }
        onPromptCaptcha: {
            captchaTimer.start()
        }
    }

    Connections {
        target: SetupWorker
        onRegistrationSuccess: {
            //: Registration complete remorse message
            //% "Registration complete!"
            setupRemorse.execute(qsTrId("whisperfish-registration-complete"), function() { console.log("Registration complete") })
        }
        onInvalidDatastore: {
            //: Failed to setup datastore error message
            //% "ERROR - Failed to setup datastore"
            setupRemorse.execute(qsTrId("whisperfish-error-invalid-datastore"), function() { console.log("Failed to setup datastore") })
        }
        onInvalidPhoneNumber: {
            //: Invalid phone number error message
            //% "ERROR - Invalid phone number registered with Signal"
            setupRemorse.execute(qsTrId("whisperfish-error-invalid-number"), function() { console.log("Invalid phone numberi registered with signal") })
        }
        onClientFailed: {
            //: Failed to setup signal client error message
            //% "ERROR - Failed to setup Signal client"
            setupRemorse.execute(qsTrId("whisperfish-error-setup-client"), function() { console.log("Failed to setup Signal client") })
        }
    }

    RemorsePopup { id: setupRemorse }

    Timer {
        id: phoneNumberTimer
        interval: 500
        running: false
        repeat: true
        onTriggered: {
            console.log("Page status: "+main.status)
            if(main.status == PageStatus.Active) {
                pageStack.push(Qt.resolvedUrl("Register.qml"))
                phoneNumberTimer.stop()
            }
        }
    }

    Timer {
        id: verifyTimer
        interval: 500
        running: false
        repeat: true
        onTriggered: {
            console.log("Page status: "+main.status)
            if(main.status == PageStatus.Active) {
                pageStack.push(Qt.resolvedUrl("Verify.qml"))
                verifyTimer.stop()
            }
        }
    }

    Timer {
        id: passwordTimer
        interval: 500
        running: false
        repeat: true
        onTriggered: {
            console.log("Page status: "+main.status)
            if(main.status == PageStatus.Active) {
                pageStack.push(Qt.resolvedUrl("Password.qml"))
                passwordTimer.stop()
            }
        }
    }

    Timer {
        id: captchaTimer
        interval: 500
        running: false
        repeat: false
        onTriggered: {
            console.log("Page status: "+main.status)
            if(main.status == PageStatus.Active) {
                pageStack.push(Qt.resolvedUrl("RegistrationCaptcha.qml"))
                CaptchaTimer.stop()
            }
        }
    }
    // <<<< TODO REMOVE WITH https://gitlab.com/rubdos/whisperfish/-/merge_requests/91
}
