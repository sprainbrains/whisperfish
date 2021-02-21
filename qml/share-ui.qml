import QtQuick 2.0
import Sailfish.Silica 1.0
import org.nemomobile.thumbnailer 1.0
import Nemo.DBus 2.0
import Sailfish.TransferEngine 1.0

Page {
    id: root

    property url source
    property variant content: ({})

    property string clientId: String(new Date().getTime())

    property bool shareDone: false

    // This page is loaded by the transfer system. We impement the following
    // procedure: When the page is ready, Whisperfish is called via dbus with
    // all relevant information to handle the sharing. When it is done,
    // Whisperfish will call us back so we can reactivate our window to not
    // interrupt the users workflow. If the user returns earlier, we pretend
    // the shareing is completed.

    Component.onCompleted: {
        whisperfishApp.call(
            "handleShare",
            [clientId, String(source), JSON.stringify(content ? content : {})],
            function () { shareDone = true },
            function (error, message) {
                console.log('Calling Whisperfish on DBus failed: ' + error + ' message: ' + message)
                spinner.running = false
                spinner.text = "Sharing Failed\n" + message
            }
        );
    }

    DBusInterface {
        id: whisperfishApp
        service: "be.rubdos.whisperfish"
        path: "/be/rubdos/whisperfish/app"
        iface: "be.rubdos.whisperfish.app"
    }

    DBusAdaptor {
        service: "be.rubdos.whisperfish.shareClient.c" + clientId
        path: "/be/rubdos/whisperfish/shareClient/c" + clientId
        iface: "be.rubdos.whisperfish.shareClient"

        function done() {
            console.log("DBus shareClient.done() call received");
            spinner.text = "success"
            pageStack.pop()
            activate()
        }
    }

    BusyLabel {
        id: spinner
        running: true
        opacity: 1
        text: "Waiting for Whisperfish"
    }

    Connections {
        target: Qt.application
        onStateChanged: {
            if(Qt.application.state == Qt.ApplicationActive && shareDone) {
                pageStack.pop()
            }
        }
    }
}
