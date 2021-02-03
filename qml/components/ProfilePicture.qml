import QtQuick 2.4
import Sailfish.Silica 1.0

MouseArea {
    id: root
    // TODO If performance is bad in large lists, we should consider
    // wrapping most of the contents in a Loader and show a placeholder.

    property bool highlighted: pressed // if the whole image is highlighted
    property bool labelsHighlighted: highlighted // if only the labels are highlighted

    property alias imageSource: image.source
    property bool isGroup
    property bool showInfoMark
    property alias infoMark: infoMarkIcon
    property real infoMarkSize: Theme.iconSizeSmall // set this, don't change infoMark.{width,height}

    // internally used to keep bindings even when changed from outside
    property bool _hasImage: imageSource !== ''
    property bool _highlighted: highlighted || pressed
    property bool _labelsHighlighted: labelsHighlighted || highlighted

    height: parent.height-3*Theme.paddingSmall
    width: height

    Rectangle {
        id: profileBackground
        anchors.fill: parent
        radius: 180
        opacity: Theme.opacityLow
        color: _highlighted ? Qt.tint(Theme.overlayBackgroundColor,
                                      Theme.highlightDimmerColor) :
                              Theme.overlayBackgroundColor
    }

    HighlightImage {
        source: isGroup ? "../../icons/icon-m-group.png" :
                          "image://theme/icon-m-contact"
        anchors.centerIn: parent
        highlighted: _labelsHighlighted
        opacity: !_hasImage || image.status !== Image.Ready ?
                     Theme.opacityLow : 0.0
        visible: opacity > 0.0
        color: Theme.secondaryColor
        highlightColor: Theme.secondaryHighlightColor
        Behavior on opacity { FadeAnimator { } }
    }

    Image {
        id: image
        layer.enabled: true
        layer.smooth: true
        visible: false
        anchors.fill: parent
        asynchronous: true
        sourceSize.width: parent.width
    }

    Rectangle {
        id: shapeMask
        anchors.fill: profileBackground
        radius: 180
        visible: false
        layer.enabled: true
    }

    Rectangle { // effect container
        anchors.fill: shapeMask
        color: "transparent"

        visible: _hasImage
        opacity: _highlighted ? Theme.opacityLow : 1.0

        Rectangle {
            id: infoMarkMask
            anchors { bottom: parent.bottom; right: parent.right }
            width: infoMarkSize; height: width
            radius: 180
            visible: showInfoMark
        }

        layer.enabled: true
        layer.samplerName: "imask"
        layer.effect: ShaderEffect {
            property variant source: image
            property variant omask: shapeMask
            fragmentShader: "
                varying highp vec2 qt_TexCoord0;
                uniform highp float qt_Opacity;
                uniform lowp sampler2D source;
                uniform lowp sampler2D imask;
                uniform lowp sampler2D omask;
                void main(void) {
                    gl_FragColor = \
                        texture2D(source, qt_TexCoord0.st) *
                        min((texture2D(omask, qt_TexCoord0.st).a),
                            (1.0-texture2D(imask, qt_TexCoord0.st).a)) *
                        qt_Opacity;
                }
            "
        }
    }

    Rectangle {
        id: infoMark
        anchors { bottom: parent.bottom; right: parent.right }
        width: infoMarkSize; height: width
        radius: 180
        visible: showInfoMark
        color: "transparent"

        HighlightImage {
            id: infoMarkIcon
            // source: 'image://theme/icon-s-checkmark' // outline looks too busy
            source: 'image://theme/icon-s-installed'
            anchors.centerIn: parent
            highlighted: _labelsHighlighted
            highlightColor: Theme.secondaryHighlightColor
        }
    }
}
