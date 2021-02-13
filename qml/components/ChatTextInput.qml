// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import QtQuick.Layouts 1.1
import Sailfish.Silica 1.0
import Sailfish.Pickers 1.0
import Nemo.Time 1.0

Item {
    id: root
    width: parent.width
    height: input.height + Theme.paddingSmall

    property alias text: input.text
    property var attachments: ([]) // always update via assignment to ensure notifications
    property alias textPlaceholder: input.placeholderText
    property alias editor: input

    // A personalized placeholder should only be shown when starting a new 1:1 chat.
    property bool enablePersonalizedPlaceholder: false
    property string placeholderContactName: ''
    property int maxHeight: 3*Theme.itemSizeLarge
    property bool showSeparator: false
    property bool clearAfterSend: true
    readonly property bool canSend: enableSending &&
                                    (text.trim().length > 0 ||
                                     attachments.length > 0)
    property bool enableSending: true

    signal sendMessage(var text, var attachments)

    function reset() {
        Qt.inputMethod.commit()
        text = ""
        attachments = []

        if (input.focus) { // reset keyboard state
            input.focus = false
            input.focus = true
        }
    }

    function forceEditorFocus(/*bool*/ atEnd) {
        if (atEnd) input.cursorPosition = input.text.length
        input.forceActiveFocus()
    }

    function _send() {
        Qt.inputMethod.commit()
        if (text.length === 0 && attachments.length === 0) return
        if(SettingsBridge.boolValue("enable_enter_send")) {
            text = text.replace(/(\r\n\t|\n|\r\t)/gm, '')
        }
        sendMessage(text, attachments)
        if (clearAfterSend) reset()
    }

    WallClock {
        id: clock
        enabled: parent.enabled && Qt.application.active
        updateFrequency: WallClock.Minute
    }

    Separator {
        opacity: showSeparator ? 1.0 : 0.0
        color: input.focus ? Theme.secondaryHighlightColor :
                             Theme.secondaryColor
        horizontalAlignment: Qt.AlignHCenter
        anchors {
            left: parent.left; leftMargin: Theme.horizontalPageMargin
            right: parent.right; rightMargin: Theme.horizontalPageMargin
            top: parent.top
        }
        Behavior on opacity { FadeAnimator { } }
    }

    RowLayout {
        width: parent.width
        height: parent.height
        layoutDirection: Qt.LeftToRight
        spacing: Theme.paddingSmall

        TextArea {
            id: input
            height: Theme.itemSizeMedium
            Layout.fillWidth: true
            Layout.fillHeight: false
            Layout.alignment: Qt.AlignLeft | Qt.AlignBottom
            Layout.maximumHeight: maxHeight
            width: parent.width - attachButton
            label: Format.formatDate(clock.time, Formatter.TimeValue) +
                   (attachments.length > 0 ?
                        " — " +
                        //: TODO
                        //% "%n attachment(s)"
                        qsTrId("whisperfish-chat-input-attachment-label", attachments.length) :
                        "")
            hideLabelOnEmptyField: false
            textRightMargin: 0
            font.pixelSize: Theme.fontSizeSmall
            placeholderText: enablePersonalizedPlaceholder && placeholderContactName.length ?
                                 //: Personalized placeholder for chat input, e.g. "Hi John"
                                 //% "Hi %1"
                                 qsTrId("whisperfish-chat-input-placeholder-personal").arg(
                                     placeholderContactName) :
                                 //: Generic placeholder for chat input
                                 //% "Message"
                                 qsTrId("whisperfish-chat-input-placeholder-default")
            EnterKey.onClicked: {
                if (canSend && SettingsBridge.boolValue("enable_enter_send")) {
                    _send()
                }
            }
        }

        IconButton {
            id: attachButton
            Layout.alignment: Qt.AlignRight
            anchors { bottom: parent.bottom; bottomMargin: Theme.paddingMedium }
            icon.source: "image://theme/icon-m-attach"
            icon.width: Theme.iconSizeMedium
            icon.height: icon.width
            onClicked: pageStack.push(contentPickerPage)
        }

        IconButton {
            id: sendButton
            Layout.alignment: Qt.AlignRight
            anchors { bottom: parent.bottom; bottomMargin: Theme.paddingMedium }
            icon.width: Theme.iconSizeMedium + 2*Theme.paddingSmall
            icon.height: width
            icon.source: "image://theme/icon-m-send"
            enabled: canSend
            onClicked: {
                if (canSend /*&& SettingsBridge.boolValue("send_on_click")*/) {
                    _send()
                }
            }
            onPressAndHold: {
                // TODO implement in backend
                if (canSend /*&& SettingsBridge.boolValue("send_on_click") === false*/) {
                    _send()
                }
            }
        }

        Component {
            id: contentPickerPage
            ContentPickerPage {
                //: Title for file picker page
                //% "Select file"
                title: qsTrId("whisperfish-select-file")
                onSelectedContentPropertiesChanged: {
                    // TODO implement selecting multiple attachments
                    root.attachments = [selectedContentProperties.filePath]
                }
            }
        }
    }
}
