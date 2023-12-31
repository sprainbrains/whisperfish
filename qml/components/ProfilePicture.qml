import QtQuick 2.4
import Sailfish.Silica 1.0

MouseArea {
    id: root
    // NOTE If performance is bad in large lists, we should consider
    // wrapping most of the contents in a Loader and show a placeholder.

    property bool highlighted: pressed // if the whole image is highlighted
    property bool labelsHighlighted: highlighted // if only the labels are highlighted

    property alias imageSource: image.source
    property alias imageStatus: image.status
    property bool isGroup
    property bool isNoteToSelf
    property bool showInfoMark
    property alias infoMarkSource: infoMark.source
    property alias infoMarkEmoji: infoMark.emoji
    property alias infoMarkRotation: infoMarkIcon.rotation
    property real infoMarkSize: Theme.iconSizeSmall // set this, don't change infoMark.{width,height}
    property real infoMarkMaskFactor: 1.2 // how much larger than the icon should the mask be?

    property color profileBackgroundColor: Theme.colorScheme === Theme.LightOnDark ?
                                               Qt.darker(Theme.secondaryHighlightColor) :
                                               (Theme.rgba(Theme.secondaryHighlightColor,
                                                           _highlighted ? Theme.opacityFaint :
                                                                          Theme.opacityLow))

    // internally used to keep bindings even when changed from outside
    property bool _hasImage: imageSource !== ''
    property bool _highlighted: highlighted || pressed
    property bool _labelsHighlighted: labelsHighlighted || _highlighted

    height: parent.height-4*Theme.paddingSmall
    width: height

    Rectangle {
        id: profileBackground
        anchors.fill: parent
        layer.enabled: true
        layer.smooth: true
        radius: width/2
        visible: false
        color: profileBackgroundColor
        opacity: (!_hasImage || image.status !== Image.Ready) ? 1.0 : 0.0
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
        radius: width/2
        visible: false
        layer.enabled: true
    }

    Rectangle { // effect container
        anchors.fill: shapeMask
        color: "transparent"
        opacity: _highlighted ? Theme.opacityLow : 1.0

        Rectangle {
            id: infoMarkMask
            anchors { bottom: parent.bottom; right: parent.right }
            width: infoMarkMaskFactor*infoMarkSize; height: width
            radius: width/2
            visible: showInfoMark
        }

        layer.enabled: true
        layer.samplerName: "imask"
        layer.effect: ShaderEffect {
            property variant source: (_hasImage && image.status === Image.Ready) ?
                                         image : profileBackground
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

    HighlightImage {
        source: isGroup ? "image://theme/icon-m-users" :
                          ( isNoteToSelf ? "image://theme/icon-m-note" : "image://theme/icon-m-contact" )
        anchors.centerIn: parent
        highlighted: _labelsHighlighted
        opacity: !_hasImage || image.status !== Image.Ready ?
                     (Theme.colorScheme === Theme.LightOnDark ?
                          Theme.opacityHigh : 1.0) : 0.0
        visible: opacity > 0.0
        color: Theme.secondaryColor
        highlightColor: Theme.highlightColor
        Behavior on opacity { FadeAnimator { } }
    }

    Rectangle {
        id: infoMark
        anchors {
            bottom: parent.bottom; bottomMargin: (infoMarkMask.width-infoMarkSize)/2
            right: parent.right; rightMargin: (infoMarkMask.width-infoMarkSize)/2
        }
        width: infoMarkSize; height: width
        radius: width/2
        visible: showInfoMark
        color: "transparent"
        property string source: 'image://theme/icon-s-installed'
        property string emoji: ''

        HighlightImage {
            id: infoMarkIcon
            visible: parent.emoji == ''
            enabled: visible
            source: parent.source
            anchors.fill: parent
            color: Theme.primaryColor
            highlighted: _labelsHighlighted
            highlightColor: Theme.secondaryHighlightColor
        }

        LinkedEmojiLabel {
            id: infoMarkEmoji
            visible: parent.emoji !== ''
            enabled: visible
            anchors.fill: parent
            horizontalAlignment: Text.AlignHCenter
            plainText: parent.emoji
            emojiSizeMult: 1.0
            font.pixelSize: height * 0.75
        }
    }
}
