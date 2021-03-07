// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.0
import Sailfish.Silica 1.0

Label {
    id: root
    property string defaultMessage: ''
    onDefaultMessageChanged: text = defaultMessage // no animation

    function toggleHint(hintMessage) {
        if (delay.running) {
            delay.stop()
        } else {
            if (_hinting) _newMessage = defaultMessage
            else _newMessage = hintMessage
            delay.restart()
        }
    }

    text: defaultMessage
    opacity: 1.0
    width: parent.width
    horizontalAlignment: Text.AlignHCenter
    fontSizeMode: Text.HorizontalFit
    font.pixelSize: Theme.fontSizeSmall
    color: Theme.highlightColor

    property bool _hinting: false
    property string _newMessage: ''

    function _doToggle() {
        if (_hinting) _hinting = false
        else _hinting = true
        hintAnim.restart()
    }

    SequentialAnimation {
        id: hintAnim
        FadeAnimator { target: root; from: 1.0; to: 0.0; duration: 30 }
        ScriptAction { script: { root.text = root._newMessage } }
        FadeAnimator { target: root; from: 0.0; to: 1.0; duration: 50 }
    }

    Timer {
        id: delay
        interval: 150
        onTriggered: parent._doToggle()
    }
}
