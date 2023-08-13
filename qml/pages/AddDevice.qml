import QtQuick 2.2
import Sailfish.Silica 1.0
import QtMultimedia 5.6
import Amber.QrFilter 1.0

Dialog {
    id: addDeviceDialog
    objectName: "addDeviceDialog"
    canAccept: false
    readonly property bool active: Qt.application.active

    onActiveChanged: {
        if(active) {
            camera.stop()
        }
        else {
            camera.start()
            camera.unlock()
        }
    }

    onStatusChanged: {
        if(status === PageStatus.Active) {
            camera.start()
            camera.unlock()
        }
        else {
            camera.stop()
        }
    }

    signal addDevice(string tsurl)

    Column {
        width: parent.width
        spacing: Theme.paddingLarge

        DialogHeader {
            acceptText: ""
            //: Add Device, shown as pull-down menu item
            //% "Add Device"
            title: qsTrId("whisperfish-add-device")
        }


        VideoOutput {
            id: videoOutput
            source: camera
            fillMode: VideoOutput.PreserveAspectFit
            z: -1
            width: parent.width - Theme.paddingLarge * 2
            height: parent.width - Theme.paddingLarge * 2
            anchors.horizontalCenter: parent.horizontalCenter

            filters: [ qrFilter ]
            
            MouseArea {
                anchors.fill: parent
                onClicked: {
                    camera.unlock()
                    camera.searchAndLock()
                }
            }
        }

        QrFilter {
            id: qrFilter
            onResultChanged: {
                if (result.length > 0 && 
                    (result.indexOf("tsdevice:") == 0 || result.indexOf("sgnl:") == 0)) {
                    addDevice(result)
                    addDeviceDialog.close()
                }
            }
        }

        Label {
            width: parent.width
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            //: Instructions on how to scan QR code for device linking
            //% "Scan the QR code displayed by the Signal application that you wish to link"
            text: qsTrId("whisperfish-qr-scanning-instructions")
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.highlightColor


            anchors {
                left: parent.left
                leftMargin: Theme.horizontalPageMargin
                right: parent.right
                rightMargin: Theme.horizontalPageMargin
            }
        }
    }

    Camera {
        id: camera
        position: Camera.BackFace
        captureMode: Camera.CaptureStillImage

        exposure {
            exposureMode: Camera.ExposureAuto
        }

        flash.mode: Camera.FlashOff

        onCameraStatusChanged: {
            if (cameraStatus === Camera.ActiveStatus) {
                var resolutions = camera.supportedViewfinderResolutions()
                var selectedResolution
                if (resolutions.length > 0) {
                    for (var i = 0; i < resolutions.length; i++) {
                        var resolution = resolutions[i]
                        // Looking for the largest square that will fit the width
                        if (resolution.height === resolution.width && resolution.width  <= Screen.width) {
                            selectedResolution = resolution
                        }
                    }
                }
                if (selectedResolution) {
                    camera.viewfinder.resolution = Qt.size(selectedResolution.width, selectedResolution.height)
                }
            }
        }
    }
}
