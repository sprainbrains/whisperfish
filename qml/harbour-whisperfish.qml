import QtQuick 2.0
import Sailfish.Silica 1.0
import "pages"

// Note: This is the main QML file for reCaptcha helper application.
// Are you looking for harbour-whisperfish-main.qml?

ApplicationWindow
{
    initialPage: Component { RegistrationCaptcha { } }
    cover: Qt.resolvedUrl("cover/RegistrationCoverPage.qml")
    allowedOrientations: defaultAllowedOrientations
}
