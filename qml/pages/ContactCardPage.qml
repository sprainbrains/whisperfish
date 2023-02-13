// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.0
import Sailfish.Silica 1.0
import org.nemomobile.contacts 1.0 as NemoContacts
import Sailfish.Contacts 1.0 as SailfishContacts

Page {
    id: root
    objectName: "contactCardPage"

    allowedOrientations: Orientation.All
    property alias vcfUrl: vcfModel.source
    property alias _contact: contactCard.contact

    function safeFileName(person) {
        var noWhitespace = person.displayLabel.replace(/\s/g, '')
        var sevenBit = Format.formatText(noWhitespace, Formatter.Ascii7Bit)
        if (sevenBit.length < noWhitespace.length) sevenBit = 'contact_file'
        return Format.formatText(sevenBit, Formatter.PortableFilename) + '.vcf'
    }

    NemoContacts.PeopleVCardModel {
        id: vcfModel
        Component.onCompleted: {
            if (count === 1) _contact = getPerson(0)
            else console.warn("[ContactCardPage] showing more than one contact is not supported")
            contactCard.refreshDetails()
        }
    }

    SailfishContacts.ContactCard {
        id: contactCard

        PullDownMenu {
            // TODO investigate why sharing crashes Whisperfish
            /* MenuItem {
                //: Menu item to share a contact card
                //% "Share"
                text: qsTrId("whisperfish-contact-card-page-share")
                onClicked: {
                    var content = {
                        'name': safeFileName(contactCard.contact),
                        'type': 'text/x-vcard',
                        'data': contactCard.contact.vCard(),
                        'icon': contactCard.contact.avatarPath.toString()
                    }
                    pageStack.animatorPush('Sailfish.Contacts.ContactSharePage', {'content': content})
                }
            } */
            MenuItem {
                //: Menu item to save a shared contact to the local address book
                //% "Save to address book"
                text: qsTrId("whisperfish-contact-card-page-save")
                onClicked: Qt.openUrlExternally(vcfUrl)
            }
        }
    }
}
