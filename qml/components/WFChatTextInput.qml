/*
 * Adpoted for use with Whisperfish 
 *
 * Copyright (C) 2012-2015 Jolla Ltd.
 *
 * The code in this file is distributed under multiple licenses, and as such,
 * may be used under any one of the following licenses:
 *
 *   - GNU General Public License as published by the Free Software Foundation;
 *     either version 2 of the License (see LICENSE.GPLv2 in the root directory
 *     for full terms), or (at your option) any later version.
 *   - GNU Lesser General Public License as published by the Free Software
 *     Foundation; either version 2.1 of the License (see LICENSE.LGPLv21 in the
 *     root directory for full terms), or (at your option) any later version.
 *   - Alternatively, if you have a commercial license agreement with Jolla Ltd,
 *     you may use the code under the terms of that license instead.
 *
 * You can visit <https://sailfishos.org/legal/> for more information
 */

/*
 * Modifications for Whisperfish:
 * SPDX-FileCopyrightText: 2021 Mirian Margiani
 * SPDX-License-Identifier: AGPL-3.0-or-later
 *
 */

import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.Pickers 1.0
import org.nemomobile.time 1.0

InverseMouseArea {
    id: chatInputArea

    // Can't use textField height due to excessive implicit padding
    height: timestamp.y + timestamp.height + Theme.paddingMedium

    readonly property bool inputFieldFocused: textField.focus

    property string contactName: ""
    property string attachmentPath: ""
    property alias text: textField.text
    property alias cursorPosition: textField.cursorPosition
    property alias editorFocus: textField.focus
    property bool enabled: true
    property bool clearAfterSend: true

    property bool canSend: text.trim().length > 0 || attachmentPath.trim().length > 0

    signal sendMessage(string text, string path)

    function setAttachmentPath(path) {
        attachmentPath = path
    }

    function send() {
        Qt.inputMethod.commit()
        if (text.length < 1 && attachmentPath.length < 1)
            return

        if(SettingsBridge.boolValue("enable_enter_send")) {
            text = text.replace(/(\r\n\t|\n|\r\t)/gm,"")
        }
        sendMessage(text, attachmentPath)
        if (clearAfterSend) {
            text = ""
            attachmentPath = ""
        }
        // Reset keyboard state
        if (textField.focus) {
            textField.focus = false
            textField.focus = true
        }
    }

    function forceActiveFocus() {
        textField.forceActiveFocus()
    }

    function reset() {
        Qt.inputMethod.commit()
        text = ""
    }

    property Page page: _findPage()
    function _findPage() {
        var parentItem = parent
        while (parentItem) {
            if (parentItem.hasOwnProperty('__silica_page')) {
                return parentItem
            }
            parentItem = parentItem.parent
        }
        return null
    }

    property bool onScreen: visible && Qt.application.active && page !== null && page.status === PageStatus.Active

    TextArea {
        id: textField
        anchors {
            left: parent.left
            right: sendButtonArea.left
            top: parent.top
            topMargin: Theme.paddingMedium
        }

        focusOutBehavior: FocusBehavior.KeepFocus
        textRightMargin: 0
        font.pixelSize: Theme.fontSizeSmall

        EnterKey.onClicked: {
            if (canSend && SettingsBridge.boolValue("enable_enter_send")) {
                chatInputArea.send()
            }
        }

        property bool empty: text.length === 0 && !inputMethodComposing

        placeholderText: contactName.length ?
        //: Personalized placeholder for chat input, e.g. "Hi John"
        //% "Hi %1"
             qsTrId("whisperfish-chatinput-contact").arg(contactName) :
        //: Generic placeholder for chat input
        //% "Hi"
             qsTrId("whisperfish-chatinput-generic")
    }

    onClickedOutside: textField.focus = false

    Row {
        id: sendButtonArea
        spacing: Theme.paddingMedium
        width: childrenRect.width
        anchors {
            right: parent.right
            rightMargin: Theme.horizontalPageMargin
            verticalCenter: textField.top
            verticalCenterOffset: textField.textVerticalCenterOffset +
                                  (textField._editor.height - (height/2))
        }
        IconButton {
            icon.source: "image://theme/icon-m-attach"
            icon.width: Theme.iconSizeMedium
            icon.height: icon.width
            onClicked: {
                chatInputArea.attachmentPath = ""
                pageStack.push(contentPickerPage)
            }
        }
        Button {
            width: Theme.iconSizeMedium + 2*Theme.paddingSmall
            height: width
            onClicked: chatInputArea.send()
            enabled: canSend
            Icon {
                width: Theme.iconSizeMedium
                height: width
                source: "image://theme/icon-m-send"
                highlighted: parent.highlighted
                anchors.centerIn: parent
                opacity: parent.enabled ? 1.0 : Theme.opacityLow
            }
        }
    }

    Label {
        id: timestamp
        anchors {
            top: textField.bottom
            // Spacing underneath separator in TextArea is _labelItem.height + Theme.paddingSmall + 3
            topMargin: -textField._labelItem.height - 3
            left: textField.left
            leftMargin: Theme.horizontalPageMargin
            right: textField.right
        }

        color: Theme.secondaryHighlightColor
        font.pixelSize: Theme.fontSizeTiny
        text: Format.formatDate(wallClock.time, Formatter.TimeValue)

        WallClock {
            id: wallClock
            enabled: Qt.application.active
            updateFrequency: WallClock.Minute
        }
    }

    Label {
        id: messageType
        anchors {
            right: parent.right
            rightMargin: Theme.horizontalPageMargin
            top: timestamp.top
        }

        color: Theme.highlightColor
        font.pixelSize: Theme.fontSizeTiny
        horizontalAlignment: Qt.AlignRight
        text: attachmentPath.length == 0 ? "" : "(1) Attachment" 
    }

    Component {
        id: contentPickerPage
        ContentPickerPage {
            //: Title for file picker page
            //% "Select file"
            title: qsTrId("whisperfish-select-file")
            onSelectedContentPropertiesChanged: {
                chatInputArea.attachmentPath = selectedContentProperties.filePath
            }
        }
    }
}
