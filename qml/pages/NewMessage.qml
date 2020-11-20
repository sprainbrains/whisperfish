/*
 * This was adapted from jolla-messages for use with Whisperfish
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

import QtQuick 2.2
import Sailfish.Silica 1.0
import Sailfish.Messages 1.0
import Sailfish.Contacts 1.0
import Sailfish.Telephony 1.0
import org.nemomobile.contacts 1.0
import org.nemomobile.commhistory 1.0

Page {
    id: newMessagePage
    property Label errorLabel

    property bool recipientSelected: recipientField.selectedContacts.count == 1
    property QtObject selectedContact: recipientSelected ? recipientField.selectedContacts.get(0) : null
    property bool isValid: recipientSelected && recipientNumber != ""

    property string recipientNumber: recipientSelected && selectedContact.propertyType == "phoneNumber" ? ContactModel.format(selectedContact.property.number) : ""
    property string recipientName: recipientSelected ? selectedContact.formattedNameText : ""

    _clickablePageIndicators: !(isLandscape && recipientField.activeFocus)

    SilicaFlickable {
        id: newMessage
        focus: true
        contentHeight: content.y + content.height
        anchors.fill: parent

        RemorsePopup { id: remorse }

        Column {
            id: content
            y: newMessagePage.isLandscape ? Theme.paddingMedium : 0
            width: newMessage.width
            Item {
                width: newMessage.width
                height: Math.max(recipientField.height, newMessage.height - textInput.height - content.y)

                Column {
                    id: recipientHeader
                    width: parent.width
                    PageHeader {
                        //: New message page title
                        //% "New message"
                        title: qsTrId("whisperfish-new-message-title")
                        visible: newMessagePage.isPortrait
                    }

                    RecipientField {
                        id: recipientField

                        actionType: Telephony.Message
                        width: parent.width
                        recentContactsCategoryMask: CommHistory.VoicecallCategory | CommHistory.VoicemailCategory
                        contactSearchModel: PeopleModel { filterType: PeopleModel.FilterNone }
                        requiredProperty: PeopleModel.PhoneNumberRequired
                        showLabel: newMessagePage.isPortrait

                        multipleAllowed: false

                        //: New message recipient label
                        //% "Recipient"
                        placeholderText: qsTrId("whisperfish-new-message-recipient")

                        //: Summary of all selected recipients, e.g. "Bob, Jane, 75553243"
                        //% "Recipient"
                        summaryPlaceholderText: qsTrId("whisperfish-new-message-recipient")

                        onSelectionChanged: {
                            console.log("Selected recipient name: " + recipientName);
                            console.log("Selected recipient number: " + recipientNumber);
                        }
                        onHasFocusChanged: if (!hasFocus) textInput.forceActiveFocus()
                    }
                }
                ErrorLabel {
                    id: errorLabel
                    visible: text.length > 0
                    anchors {
                        bottom: parent.bottom
                        bottomMargin: -Theme.paddingSmall
                    }
                }
            }

            // XXX consider using Sailfish's ChatTextInput
            WFChatTextInput {
                id: textInput
                width: parent.width
                enabled: recipientNumber.length != 0
                clearAfterSend: recipientNumber.length != 0

                onSendMessage: {
                    if (recipientNumber.length != 0) {
                        var source = recipientNumber
                        // Errors should be handled asynchronously
                        MessageModel.createMessage(source, text, "", "", false)
                        SessionModel.reload()
                        pageStack.pop()
                    } else {
                        //: Invalid recipient error
                        //% "Invalid recipient"
                        errorLabel.text = qsTrId("whisperfish-error-invalid-recipient")
                    }
                }
            }
        }
        VerticalScrollDecorator {}
    }
}
