import QtQuick 2.6
import Sailfish.Silica 1.0
import QtMultimedia 5.6
import QtSensors 5.0

Dialog {
    id: dialog
    allowedOrientations: Orientation.All
    property string fileName: ""
    property string fileType: ""
    property string discardedFile: ""

    readonly property string photoPath: SettingsBridge.camera_dir
    readonly property bool active: Qt.application.active

    canAccept: fileName.length > 0

    onRejected: delete_discarded_file()

    onActiveChanged: {
        if(active) {
            camera.stop()
        }
        else {
            camera.start()
        }
    }

    onStatusChanged: {
        if(status === PageStatus.Active) {
            camera.start()
        }
        else {
            camera.stop()
        }
    }

    function delete_discarded_file() {
        if(discardedFile.length > 0) {
            ClientWorker.delete_file(discardedFile)
            discardedFile = "";
        }
    }

    OrientationSensor {
        id: orientationSensor
        active: true
        onReadingChanged: {
            if(reading.orientation === OrientationReading.LeftUp) {
                camera.metaData.orientation = 180
                captureView.rotation = 90
            }
            else if(reading.orientation === OrientationReading.TopUp) {
                camera.metaData.orientation = 270
                captureView.rotation = 0
            }
            else if(reading.orientation === OrientationReading.RightUp) {
                camera.metaData.orientation = 0
                captureView.rotation = 270
            }
            else if(reading.orientation === OrientationReading.TopDown) {
                camera.metaData.orientation = 90
                captureView.rotation = 180
            }
        }
    }

    // The viewfinder
    VideoOutput {
        id: captureView
        source: camera
        width: Screen.width
        height: Screen.height
        fillMode: VideoOutput.PreserveAspectFit
        anchors.centerIn: parent
        z: -1

        MouseArea {
            anchors.fill: parent
            onClicked: {
                camera.unlock()
                camera.searchAndLock()
            }
        }
    }

    Image {
        id: photoPreview
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        visible: false

        // Obey the orientation metadata
        autoTransform: true
    }

    // The buttons
    Item {
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
        }
        height: shutterButton.height * 2

        // Show either the shutter button...

        IconButton {
            id: shutterButton
            icon.source: "image://theme/icon-m-camera?" + (pressed
                  ? Theme.highlightColor
                  : Theme.primaryColor)
            onClicked: camera.imageCapture.captureToLocation(photoPath + "/Photo_" + Qt.formatDateTime(new Date(), "yyyyMMdd_hhmmss") + ".jpg")
            anchors.horizontalCenter: parent.horizontalCenter
            enabled: camera.imageCapture.ready && photoPath.length > 0
            visible: !photoPreview.visible
            Rectangle {
                anchors.fill: parent
                radius: Math.min(parent.width, parent.height, width/2, height/2)
                color: Theme.rgba(Theme.primaryColor, 0.25)
                z: -1
            }
       }

        // ...or the picture action buttons

        IconButton {
            id: previewButton
            icon.source: "image://theme/icon-m-delete?" + (pressed
                  ? Theme.highlightColor
                  : Theme.primaryColor)
            onClicked: {
                // TODO: Delete the discarded photo/video, but how?
                // QML doesn't support fs actions...
                photoPreview.source = ""
                photoPreview.visible = false
                dialog.fileName = ""
                dialog.fileType = ""
                camera.start()
                delete_discarded_file()
            }

            anchors {
                right: parent.horizontalCenter
                rightMargin: width / 2
            }
            enabled: photoPreview.visible
            visible: photoPreview.visible

            Rectangle {
                anchors.fill: parent
                radius: Math.min(parent.width, parent.height, width/2, height/2)
                color: Theme.rgba(Theme.primaryColor, 0.25)
                z: -1
            }
        }

        IconButton {
            id: shareButton
            icon.source: "image://theme/icon-m-accept?" + (pressed
                  ? Theme.highlightColor
                  : Theme.primaryColor)
            onClicked: {
                dialog.accept()
            }

            anchors {
                left: parent.horizontalCenter
                leftMargin: width / 2
            }
            enabled: photoPreview.visible
            visible: photoPreview.visible

            Rectangle {
                anchors.fill: parent
                radius: Math.min(parent.width, parent.height, width/2, height/2)
                color: Theme.rgba(Theme.primaryColor, 0.25)
                z: -1
            }
        }
    }

    Camera {
        id: camera

        Component.onCompleted: {
            camera.start()
            camera.unlock()
        }

        exposure {
            exposureMode: Camera.ExposureAuto

        }

        flash.mode: Camera.FlashOff

        imageCapture {
            onImageSaved: {
                photoPreview.source = path
                dialog.fileName = path
                // Deleted later if necessary
                dialog.discardedFile = path
                dialog.fileType = "image/jpeg"
                photoPreview.visible = true
                camera.stop()
            }
        }
    }
}
