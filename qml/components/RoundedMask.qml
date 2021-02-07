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
    readonly property int topLeft: 1
    readonly property int topRight: 2
    readonly property int bottomLeft: 4
    readonly property int bottomRight: 8

    property int roundedCorners: 0 // e.g. topLeft | bottomLeft
    property real radius: 20

    maskSource: Item {
        width: root.width
        height: root.height

        Rectangle {
            id: allRounded
            anchors.fill: parent
            radius: root.radius
        }
        Rectangle {
            height: ((roundedCorners & topLeft) == 0) ? 2*root.radius : 0
            width: height
            anchors { left: parent.left; top: parent.top }
        }
        Rectangle {
            height: ((roundedCorners & topRight) == 0) ? 2*root.radius : 0
            width: height
            anchors { right: parent.right; top: parent.top }
        }
        Rectangle {
            height: ((roundedCorners & bottomLeft) == 0) ? 2*root.radius : 0
            width: height
            anchors { left: parent.left; bottom: parent.bottom }
        }
        Rectangle {
            height: ((roundedCorners & bottomRight) == 0) ? 2*root.radius : 0
            width: height
            anchors { right: parent.right; bottom: parent.bottom }
        }
    }
}
