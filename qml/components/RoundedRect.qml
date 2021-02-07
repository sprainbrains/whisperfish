// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.2

/*!
  This component is a rectangle with separately rounded corners.

  Set the \c roundedCorners property to an OR-combination of
  the four corners: \c topLeft, \c topRight, \c bottomLeft, \c bottomRight.
*/
Item {
    id: root
    readonly property int topLeft: 1
    readonly property int topRight: 2
    readonly property int bottomLeft: 4
    readonly property int bottomRight: 8

    property int roundedCorners: 0 // e.g. topLeft | bottomLeft
    property color color: "white"
    property real radius: 10

    ShaderEffect {
        property alias color: root.color
        property var source: ShaderEffectSource {
            sourceRect: Qt.rect(0, 0, root.width, root.height)
            sourceItem: Rectangle {
                width: root.width
                height: root.height
                radius: root.radius
            }
        }

        property bool rTL: (roundedCorners & topLeft) > 0
        property bool rTR: (roundedCorners & topRight) > 0
        property bool rBL: (roundedCorners & bottomLeft) > 0
        property bool rBR: (roundedCorners & bottomRight) > 0

        anchors.fill: parent
        fragmentShader: "
            uniform sampler2D source;
            varying highp vec2 qt_TexCoord0;
            uniform highp vec4 color;
            uniform highp float qt_Opacity;

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
                    gl_FragColor = color * (texture2D(source, qt_TexCoord0).w) * qt_Opacity;
                } else {
                    gl_FragColor = color * qt_Opacity;
                }
            }
        "
    }
}
