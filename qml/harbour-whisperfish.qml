import QtQuick 2.2
import Sailfish.Silica 1.0
import Nemo.Notifications 1.0
import Nemo.DBus 2.0
import Nemo.Configuration 1.0
import org.nemomobile.contacts 1.0
import "pages"
import "js/emoji.js" as Emoji // for initialization

ApplicationWindow
{
    id: mainWindow
    cover: Qt.resolvedUrl("cover/CoverPage.qml")
    initialPage: Component { LandingPage { } }
    allowedOrientations: Orientation.All
    _defaultPageOrientations: Orientation.All
    _defaultLabelFormat: Text.PlainText
    Component.onCompleted: Emoji.dataBaseDirectory = StandardPaths.genericData

    readonly property string mainPageName: "mainPage"
    readonly property string conversationPageName: "conversationPage"
    property var notificationMap: ({})

    // TODO This should be moved to a singleton Emoji type,
    // together with the js import and data directory initialization.
    readonly property string emojiStyle: globalEmojiStyleConfig.value

    // setting this to "true" will block global navigation
    // methods (showMainPage() etc.)
    property bool fatalOccurred: false

    property alias contactsReady: resolvePeopleModel.populated

    PeopleModel {
        id: resolvePeopleModel

        // Specify the PhoneNumberRequired flag to ensure that all phone number
        // data will be loaded before the model emits populated.
        // This ensures that we resolve numbers to contacts appropriately, in
        // the case where we attempt to message a newly-created contact via
        // the action shortcut icon in the contact card.
        requiredProperty: PeopleModel.PhoneNumberRequired
    }

    Component {
        id: messageNotification
        Notification {}
    }

    ConfigurationValue {
        id: globalEmojiStyleConfig
        // TODO FIXME No signal is fired when the value changes from
        // outside of Whisperfish, e.g. when the current style is changed
        // in the keyboard config popup. This may be related to #140.
        key: '/apps/sailor-emoji/currentStyle'
        defaultValue: 'openmoji'
    }

    function activateSession(sid, name, source) {
        console.log("Activating session for source: "+source)
        MessageModel.load(sid, name)
    }

    function newMessageNotification(sid, sessionName, senderIdentifier, message, isGroup) {
        var contact = resolvePeopleModel.personByPhoneNumber(senderIdentifier);
        var name = (isGroup || !contact) ? sessionName : contact.displayLabel;
        var contactName = contact ? contact.displayLabel : senderIdentifier;

        if(Qt.application.state == Qt.ApplicationActive &&
           (pageStack.currentPage.objectName == mainPageName ||
           (sid == MessageModel.sessionId && pageStack.currentPage.objectName == conversationPageName))) {
           return
        }

        var m
        if(SettingsBridge.boolValue("show_notify_message")) {
            m = messageNotification.createObject(null)
            m.body = message
            m.itemCount = 1
        } else {
            if (sid in notificationMap) {
                m = notificationMap[sid][0]
                m.itemCount++
            } else {
                m = messageNotification.createObject(null)
                m.itemCount = 1
            }
            //: Default label for new message notification
            //% "New Message"
            m.body = qsTrId("whisperfish-notification-default-message")
        }

        if (SettingsBridge.boolValue("minimise_notify") && (sid in notificationMap)) {
            var first_message = notificationMap[sid][0]
            m.replacesId = first_message.replacesId
            m.itemCount = first_message.itemCount + 1
        }

        m.category = "harbour-whisperfish-message"
        m.previewSummary = name
        m.previewBody = m.body
        m.summary = name
        m.subText = contactName
        m.clicked.connect(function() {
            console.log("Activating session: "+sid)
            mainWindow.activate()
            showMainPage()
            mainWindow.activateSession(sid, name, sessionName)
            pageStack.push(Qt.resolvedUrl("pages/ConversationPage.qml"), {}, PageStackAction.Immediate)
        })
        // This is needed to call default action
        m.remoteActions = [ {
            "name": "default",
            "displayName": "Show Conversation",
            "icon": "harbour-whisperfish",
            "service": "org.whisperfish.session",
            "path": "/message",
            "iface": "org.whisperfish.session",
            "method": "showConversation",
            "arguments": [ "sid", sid ]
        } ]
        m.publish()
        if(sid in notificationMap) {
              notificationMap[sid].push(m)
        } else {
              notificationMap[sid] = [m]
        }
    }

    Connections {
        target: ClientWorker
        onMessageReceived: {
            if(sid == MessageModel.sessionId && pageStack.currentPage.objectName == conversationPageName) {
                SessionModel.add(sid, true)
                MessageModel.add(mid)
            } else {
                SessionModel.add(sid, false)
            }
        }
        onMessageReceipt: {
            if(mid > 0 && pageStack.currentPage.objectName == conversationPageName) {
                MessageModel.markReceived(mid)
            }

            if(sid > 0) {
                SessionModel.markReceived(sid)
            }
        }
        onNotifyMessage: {
            newMessageNotification(sid, sessionName, senderIdentifier, message, isGroup)
        }
        onMessageSent: {
            if(sid == MessageModel.sessionId && pageStack.currentPage.objectName == conversationPageName) {
                SessionModel.markSent(sid, message)
                MessageModel.markSent(mid)
            } else {
                SessionModel.markSent(sid, message)
            }
        }
        onPromptResetPeerIdentity: {
            if (fatalOccurred) return
            pageStack.push(Qt.resolvedUrl("pages/PeerIdentityChanged.qml"), { source: source })
        }
    }

    Connections {
        target: SetupWorker
        onClientFailed: {
            console.log("[FATAL] client failed")
            //: Failed to setup signal client error message
            //% "Failed to setup Signal client"
            showFatalError(qsTrId("whisperfish-fatal-error-setup-client"))
        }
        onInvalidDatastore: {
            //: Failed to setup datastore error message
            //% "Failed to setup data storage"
            showFatalError(qsTrId("whisperfish-fatal-error-invalid-datastore"))
        }
    }

    Connections {
        target: Qt.application
        onStateChanged: {
            if(Qt.application.state == Qt.ApplicationActive) {
                AppState.setActive()
            }
        }
    }

    Connections {
        target: AppState
        onActivate: mainWindow.activate()
    }

    DBusAdaptor {
        service: "be.rubdos.whisperfish.app"
        path: "/be/rubdos/whisperfish"
        iface: "be.rubdos.whisperfish.app"

        function show() {
            console.log("DBus app.show() call received")
            if(Qt.application.state == Qt.ApplicationActive) {
                return
            }

            mainWindow.activate()
            if (AppState.isClosed()) {
                showMainPage()
            }
        }
    }

    function clearNotifications(sid) {
        // Close out any existing notifications for the session
        if(sid in notificationMap) {
            for(var i = 0; i < notificationMap[sid].length; i++) {
                notificationMap[sid][i].close()
            }
            delete notificationMap[sid]
        }
    }

    function showFatalError(message) {
        fatalOccurred = true
        // We don't clear the stack to keep transition animations
        // clean. FatalErrorPage will block any further navigation.
        pageStack.push(Qt.resolvedUrl("pages/FatalErrorPage.qml"), {
                           errorMessage: message
                       })
    }

    function showMainPage(operationType) {
        if (fatalOccurred) return
        pageStack.replaceAbove(null, Qt.resolvedUrl("pages/MainPage.qml"), {},
                               operationType !== undefined ? operationType :
                                                             PageStackAction.Immediate)
    }

    function newMessage(operationType) {
        if (fatalOccurred) return
        showMainPage()
        pageStack.push(Qt.resolvedUrl("pages/NewMessage.qml"), { }, operationType)
    }
}
