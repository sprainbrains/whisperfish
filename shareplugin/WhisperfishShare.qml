import QtQuick 2.6
import Sailfish.Silica 1.0
import Nemo.Thumbnailer 1.0
import Nemo.DBus 2.0
import Sailfish.TransferEngine 1.0

SilicaFlickable {
    id: root

    property var shareAction

    width: Screen.width
    height: Screen.height/2

    property string clientId: String(new Date().getTime())

    // This page is loaded by the transfer system. We implement the following
    // procedure: When the page is ready, Whisperfish is called via dbus with
    // all relevant information to handle the sharing. When it is done,
    // Whisperfish will call us back so we can reactivate our window to not
    // interrupt the user's workflow. If the user returns earlier, we pretend
    // the sharing is completed.

    Component.onCompleted: {
        whisperfishApp.call(
            "handleShare",
            [clientId, shareAction.toConfiguration()],
            function () { },
            function (error, message) {
                console.log('Calling Whisperfish on DBus failed: ' + error + ' message: ' + message)
                spinner.running = false
                spinner.text = "Sharing failed\n" + message
            }
        )
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
            spinner.text = "Sharing complete"
            spinner.running = false
            activate()
            shareAction.done()
            // TODO: How to dismiss the sharing dialog?
        }
    }

    BusyLabel {
        id: spinner
        anchors.centerIn: parent
        running: true
        opacity: running ? 1 : 0
        text: "Waiting for Whisperfish"
    }

    Label {
        opacity: spinner.running ? 0 : 1
        anchors.centerIn: parent
        text: "Sharing complete"
        font.pixelSize: Theme.fontSizeExtraLarge
    }
}
