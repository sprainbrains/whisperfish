// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.2

/*!
  This component is a rectangle with separately rounded corners.

  Set the \c roundedCorners property to an OR-combination of
  the four corners: \c topLeft, \c topRight, \c bottomLeft, \c bottomRight.

  This has better performance than using a Rectangle with
  layering and a RoundedMask.

  The corner effect has a maximum size of 5000px in either dimension.
  Larger instances will be rendered like a normal rectangle,
  with all four corners rounded.
*/
Item {
    id: root
    readonly property int none: 0
    readonly property int topLeft: 1
    readonly property int topRight: 2
    readonly property int bottomLeft: 4
    readonly property int bottomRight: 8
    readonly property int allCorners: topLeft | topRight | bottomLeft | bottomRight

    property int roundedCorners: 0 // e.g. topLeft | bottomLeft
    property color color: "white"
    property real radius: 10

    property color borderColor: "transparent"
    property int borderWidth: 0

    readonly property int maximumSize: 5000
    readonly property bool isOversized: width > maximumSize || height > maximumSize

    Loader {
        id: rectLoader
        sourceComponent: isOversized ? rectComponent : undefined
        asynchronous: true
        visible: isOversized
    }

    Component {
        id: rectComponent
        Rectangle {
            width: root.width
            height: root.height
            radius: root.radius
            color: root.color
            border.width: borderWidth
            border.color: borderColor
        }
    }

    ShaderEffect {
        enabled: !isOversized
        visible: enabled
        blending: false

        property color color: root.color
        property color borderColor: root.borderColor
        property var source: ShaderEffectSource {
            enabled: !isOversized
            sourceRect: Qt.rect(0, 0, root.width, root.height)
            sourceItem: Rectangle {
                width: root.width
                height: root.height
                radius: root.radius
                color: root.color
                border.width: borderWidth
                border.color: borderColor
            }
        }

        property bool rTL: (roundedCorners & topLeft) > 0
        property bool rTR: (roundedCorners & topRight) > 0
        property bool rBL: (roundedCorners & bottomLeft) > 0
        property bool rBR: (roundedCorners & bottomRight) > 0
        property real borderW: borderWidth/root.width
        property real borderH: borderWidth/root.height

        anchors.fill: parent
        fragmentShader: "
            uniform sampler2D source;
            varying highp vec2 qt_TexCoord0;
            uniform highp vec4 color;
            uniform highp vec4 borderColor;
            uniform highp float qt_Opacity;
            uniform highp float borderW;
            uniform highp float borderH;

            uniform bool rTL;
            uniform bool rTR;
            uniform bool rBL;
            uniform bool rBR;

            void main() {
                if (   (rTL && qt_TexCoord0.x <  0.5 && qt_TexCoord0.y <  0.5)
                    || (rTR && qt_TexCoord0.x >= 0.5 && qt_TexCoord0.y <  0.5)
                    || (rBL && qt_TexCoord0.x <  0.5 && qt_TexCoord0.y >= 0.5)
                    || (rBR && qt_TexCoord0.x >= 0.5 && qt_TexCoord0.y >= 0.5)
                ) {
                    gl_FragColor = texture2D(source, qt_TexCoord0.st) * qt_Opacity;
                } else if (   qt_TexCoord0.x <= borderW || (1.0-qt_TexCoord0.x) <= borderW
                           || qt_TexCoord0.y <= borderH || (1.0-qt_TexCoord0.y) <= borderH
                ) {
                    gl_FragColor = borderColor * qt_Opacity;
                } else {
                    gl_FragColor = color * qt_Opacity;
                }
            }
        "
    }
}
