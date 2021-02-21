// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.2
import QtGraphicalEffects 1.0

/*!
  This component provides a way to separately round corners of an item.

  Set the \c roundedCorners property to an OR-combination of
  the four corners: \c topLeft, \c topRight, \c bottomLeft, \c bottomRight.

  Set the \c radius property to the radius for all corners.

  Example:

  \qml
    Image {
        width: 100; height: 100
        fillMode: Image.Stretch
        source: "../pic1.png"
        layer.enabled: true
        layer.effect: RoundedMask {
            roundedCorners: topLeft | bottomLeft | bottomRight
            radius: 25
        }
    }
  \endqml

  If you only want to show a simple Rectangle, then prefer
  the \c RoundedRect component. It has better performance.
*/
OpacityMask {
    id: root
    readonly property alias topLeft: src.topLeft
    readonly property alias topRight: src.topRight
    readonly property alias bottomLeft: src.bottomLeft
    readonly property alias bottomRight: src.bottomRight
    readonly property alias allCorners: src.allCorners
    readonly property alias none: src.none

    property alias roundedCorners: src.roundedCorners // e.g. topLeft | bottomLeft
    property alias radius: src.radius

    maskSource: RoundedRect {
        id: src
        width: root.width
        height: root.height
    }
}
