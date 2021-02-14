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
    height: column.height + Theme.paddingSmall

    property alias text: input.text
    property var attachments: ([]) // always update via assignment to ensure notifications
    property alias textPlaceholder: input.placeholderText
    property alias editor: input

    // A personalized placeholder should only be shown when starting a new 1:1 chat.
    property bool enablePersonalizedPlaceholder: false
    property string placeholderContactName: ''
    property int maxHeight: 3*Theme.itemSizeLarge // TODO adapt based on screen size
    property bool showSeparator: false
    property bool clearAfterSend: true
    property bool enableSending: true

    readonly property var quotedMessageData: _quotedMessageData // change via setQuote()/resetQuote()
    readonly property int quotedMessageIndex: _quotedMessageIndex // change via setQuote()/resetQuote()
    readonly property bool quotedMessageShown: quotedMessageData !== null
    readonly property bool canSend: enableSending &&
                                    (text.trim().length > 0 ||
                                     attachments.length > 0)

    property var _quotedMessageData: null
    property int _quotedMessageIndex: -1

    signal sendMessage(var text, var attachments, var replyTo)
    signal quotedMessageClicked(var index, var modelData)

    function reset() {
        Qt.inputMethod.commit()
        text = ""
        attachments = []
        resetQuote()

        if (input.focus) { // reset keyboard state
            input.focus = false
            input.focus = true
        }
    }

    function setQuote(index, modelData) {
        _quotedMessageIndex = index
        _quotedMessageData = {
            message: modelData.message,
            source: modelData.source,
            outgoing: modelData.outgoing,
        }
    }

    function resetQuote() {
        _quotedMessageIndex = -1
        _quotedMessageData = null
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

    Column {
        id: column
        width: parent.width
        height: input.height + spacing + quoteItem.height
        spacing: Theme.paddingSmall

        BackgroundItem {
            id: quoteItem
            width: parent.width - 2*Theme.horizontalPageMargin
            anchors.horizontalCenter: parent.horizontalCenter
            height: quotedMessageData === null ? 0 : quoteColumn.height
            visible: height > 0
            _backgroundColor: "transparent"
            clip: true
            onClicked: quotedMessageClicked(quotedMessageIndex, quotedMessageData)

            Behavior on height { SmoothedAnimation { duration: 120 } }

            Column {
                id: quoteColumn
                spacing: Theme.paddingSmall
                height: childrenRect.height
                width: parent.width

                Item { height: 1; width: parent.width } // spacing

                Label {
                    width: parent.width
                    maximumLineCount: 1
                    truncationMode: TruncationMode.Fade
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignLeft
                    text: quotedMessageData !== null ?
                              (quotedMessageData.outgoing ?
                                  //: TODO
                                  //% "You"
                                  qsTrId("whisperfish-chat-input-quoted-message-title-outgoing") :
                                  ContactModel.name(quotedMessageData.source)) :
                              ''
                    font.pixelSize: Theme.fontSizeExtraSmall
                    font.bold: true
                    color: quotedMessageData !== null ?
                               Qt.tint(quoteItem.highlighted ? Theme.highlightColor : Theme.primaryColor,
                                       '#'+Qt.md5(quotedMessageData.source).substr(0, 6)+'0F') :
                               Theme.secondaryHighlightColor

                    IconButton {
                        id: closeReplyButton
                        anchors {
                            verticalCenter: parent.verticalCenter
                            right: parent.right
                        }
                        width: 1.5*Theme.iconSizeSmall
                        height: width
                        icon.source: "image://theme/icon-s-clear-opaque-cross"
                        onClicked: resetQuote()
                    }
                }

                LinkedEmojiLabel {
                    width: parent.width
                    verticalAlignment: Text.AlignTop
                    horizontalAlignment: Text.AlignLeft
                    plainText: quotedMessageData !== null ? quotedMessageData.message : ''
                    maximumLineCount: 2
                    // height: maximumLineCount*font.pixelSize
                    // enableElide: Text.ElideRight -- no elide to enable dynamic height
                    font.pixelSize: Theme.fontSizeExtraSmall
                    color: quoteItem.highlighted ? Theme.secondaryHighlightColor :
                                                   Theme.secondaryColor
                }
            }
        }

        RowLayout {
            width: parent.width
            layoutDirection: Qt.LeftToRight
            spacing: Theme.paddingSmall
            Layout.fillHeight: true

            TextArea {
                id: input
                Layout.minimumHeight: Theme.itemSizeMedium
                Layout.fillWidth: true
                Layout.fillHeight: false
                Layout.alignment: Qt.AlignLeft | Qt.AlignBottom
                Layout.maximumHeight: maxHeight - column.spacing - quoteItem.height
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
}
