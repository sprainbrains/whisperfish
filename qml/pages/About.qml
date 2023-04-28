import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: aboutpage
    objectName: "aboutPage"

    SilicaFlickable {
        anchors.fill: parent
        contentWidth: parent.width
        contentHeight: col.height + Theme.paddingLarge

        VerticalScrollDecorator {}

        Column {
            id: col
            spacing: Theme.paddingLarge
            width: parent.width
            PageHeader {
                //: Title for about page
                //% "About Whisperfish"
                title: qsTrId("whisperfish-about")
            }

            Image {
                anchors.horizontalCenter: parent.horizontalCenter
                source: "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
            }

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                font.bold: true
                //: Whisperfish version string
                //% "Whisperfish v%1"
                text: qsTrId("whisperfish-version").arg(Qt.application.version)
            }

            TextArea {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                horizontalAlignment: TextEdit.Center
                readOnly: true
                //: Whisperfish description
                //% "Signal client for Sailfish OS"
                text: qsTrId("whisperfish-description")
            }

            SectionHeader {
                //: Description
                //% "Description"
                text: qsTrId("whisperfish-description-section")
            }

            TextArea {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                horizontalAlignment: TextEdit.Center
                readOnly: true
                //: Whisperfish description, longer version, also for Jolla Store
                //% "Whisperfish is an unofficial, but advanced Signal client for Sailfish OS. "
                //% "Whisperfish is highly usable, but is still considered beta quality software. "
                //% "Make sure to update regularily! Also, check our Wiki and feel free to contribute to it! "
                //% "Do not ever contact the Signal developers about a Whisperfish issue, contact us instead!."
                text: qsTrId("whisperfish-long-description")
            }

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                horizontalAlignment: TextEdit.Center
                wrapMode: Text.Wrap
                text: {
                    var build_id = CiJobUrl ?
                        "<a href=\"" + CiJobUrl + "\">" + LongAppVersion + "</a>"
                        : LongAppVersion ;
                    //: Whisperfish long version string and build ID
                    //% "Build ID: %1"
                    qsTrId("whisperfish-build-id").arg(build_id)
                }
                textFormat: Text.StyledText
                onLinkActivated: Qt.openUrlExternally(link)
                linkColor: Theme.primaryColor
            }

            SectionHeader {
                //: Copyright
                //% "Copyright"
                text: qsTrId("whisperfish-copyright")
            }

            TextArea {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                horizontalAlignment: TextEdit.Center
                readOnly: true
                text: {
                    "Andrew E. Bruno (2016-2018)\n" +
                    "Ruben De Smet (2019-2022)\n" +
                    "Matti \"direc85\" Viljanen (2021-2022)\n" +
                    "Markus Törnqvist (2019-2021)\n" +
                    "Mirian Margiani (2021)\n" +
                    "Gabriel Margiani (2021)\n" +
                    "Thomas Michel (2021)\n" +
                    "License: AGPLv3 & GPLv3"
                }
            }

            SectionHeader {
                //: Translators heading in About page
                //% "Translators"
                text: qsTrId("whisperfish-translators")
            }

            TextArea {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                horizontalAlignment: TextEdit.Center
                readOnly: true
                /// BEGIN TRANSLATORS
                text: {
                    //: Name of the Norwegian Bokmål (nb_NO) language, about page translation section
                    //% "Norwegian Bokmål"
                    qsTrId("whisperfish-lang-nb_NO") + ": " +
                    "Allan Nordhøy," +
                    "\n" +
                    //: Name of the Dutch (Belgium) (nl_BE) language, about page translation section
                    //% "Dutch (Belgium)"
                    qsTrId("whisperfish-lang-nl_BE") + ": " +
                    "Nathan," +
                    "Alexander Schlarb," +
                    "J. Lavoie," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Lithuanian (lt) language, about page translation section
                    //% "Lithuanian"
                    qsTrId("whisperfish-lang-lt") + ": " +
                    "Gediminas Murauskas," +
                    "\n" +
                    //: Name of the Dutch (nl) language, about page translation section
                    //% "Dutch"
                    qsTrId("whisperfish-lang-nl") + ": " +
                    "Nathan," +
                    "Alexander Schlarb," +
                    "J. Lavoie," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Italian (it) language, about page translation section
                    //% "Italian"
                    qsTrId("whisperfish-lang-it") + ": " +
                    "J. Lavoie," +
                    "\n" +
                    //: Name of the German (de) language, about page translation section
                    //% "German"
                    qsTrId("whisperfish-lang-de") + ": " +
                    "J. Lavoie," +
                    "PawelSpoon," +
                    "Yo," +
                    "Stephan Lohse," +
                    "Alexander Schlarb," +
                    "Sebastian Maus," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the French (fr) language, about page translation section
                    //% "French"
                    qsTrId("whisperfish-lang-fr") + ": " +
                    "J. Lavoie," +
                    "Alexander Schlarb," +
                    "Bérenger," +
                    "Thibaut Vandervelden," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Finnish (fi) language, about page translation section
                    //% "Finnish"
                    qsTrId("whisperfish-lang-fi") + ": " +
                    "J. Lavoie," +
                    "jmcwine," +
                    "Tuomas F Nyqvist," +
                    "Alexander Schlarb," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Slovenian (sl) language, about page translation section
                    //% "Slovenian"
                    qsTrId("whisperfish-lang-sl") + ": " +
                    "Jože Prijatelj," +
                    "\n" +
                    //: Name of the Swedish (sv) language, about page translation section
                    //% "Swedish"
                    qsTrId("whisperfish-lang-sv") + ": " +
                    "Tuomas F Nyqvist," +
                    "fluffysfriends," +
                    "Luna Jernberg," +
                    "\n" +
                    //: Name of the Polish (pl) language, about page translation section
                    //% "Polish"
                    qsTrId("whisperfish-lang-pl") + ": " +
                    "A," +
                    "Alexander Schlarb," +
                    "Karol Kurek," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Chinese (Simplified) (zh_CN) language, about page translation section
                    //% "Chinese (Simplified)"
                    qsTrId("whisperfish-lang-zh_CN") + ": " +
                    "dashinfantry," +
                    "Alexander Schlarb," +
                    "Rui Kon," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Hungarian (hu) language, about page translation section
                    //% "Hungarian"
                    qsTrId("whisperfish-lang-hu") + ": " +
                    "Sz. G," +
                    "Alexander Schlarb," +
                    "carlosgonz," +
                    "\n" +
                    //: Name of the Portuguese (Brazil) (pt_PT) language, about page translation section
                    //% "Portuguese (Brazil)"
                    qsTrId("whisperfish-lang-pt_PT") + ": " +
                    "Júlio," +
                    "Antonio Maretzek," +
                    "J. Lavoie," +
                    "ssantos," +
                    "\n" +
                    //: Name of the Czech (cs) language, about page translation section
                    //% "Czech"
                    qsTrId("whisperfish-lang-cs") + ": " +
                    "PawelSpoon," +
                    "\n" +
                    //: Name of the Catalan (ca) language, about page translation section
                    //% "Catalan"
                    qsTrId("whisperfish-lang-ca") + ": " +
                    "Jaume," +
                    "\n" +
                    //: Name of the Russian (ru) language, about page translation section
                    //% "Russian"
                    qsTrId("whisperfish-lang-ru") + ": " +
                    "Николай Синёв," +
                    "\n" +
                    //: Name of the Portuguese (Brazil) (pt_BR) language, about page translation section
                    //% "Portuguese (Brazil)"
                    qsTrId("whisperfish-lang-pt_BR") + ": " +
                    "J. Lavoie," +
                    "Caio 2k," +
                    "\n" +
                    //: Name of the Romanian (ro) language, about page translation section
                    //% "Romanian"
                    qsTrId("whisperfish-lang-ro") + ": " +
                    "Florin Voicu," +
                    "\n" +
                    //: Name of the Turkish (tr) language, about page translation section
                    //% "Turkish"
                    qsTrId("whisperfish-lang-tr") + ": " +
                    "Oğuz Ersen," +
                    "\n" +
                    //: Name of the Greek (el) language, about page translation section
                    //% "Greek"
                    qsTrId("whisperfish-lang-el") + ": " +
                    "Chris," +
                    "J. Lavoie," +
                    "\n" +
                    //: Name of the Basque (eu) language, about page translation section
                    //% "Basque"
                    qsTrId("whisperfish-lang-eu") + ": " +
                    "Sergio Varela," +
                    "\n" +
                    //: Name of the Spanish (es) language, about page translation section
                    //% "Spanish"
                    qsTrId("whisperfish-lang-es") + ": " +
                    "gallegonovato," +
                    "J. Lavoie," +
                    "Jose Manuel REGUEIRA," +
                    "Alexander Schlarb," +
                    "PawelSpoon," +
                    "carlosgonz," +
                    "Allan Nordhøy," +
                    "\n" +
                    ""
                }
                /// END TRANSLATORS
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                //: Support on Liberapay
                //% "Support on Liberapay"
                text: qsTrId("whisperfish-liberapay")
                onClicked: {
                    Qt.openUrlExternally("https://liberapay.com/rubdos/")
                }
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                //: Source Code
                //% "Source Code"
                text: qsTrId("whisperfish-source-code")
                onClicked: {
                    Qt.openUrlExternally("https://gitlab.com/whisperfish/whisperfish")
                }
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                //: Report a Bug
                //% "Report a Bug"
                text: qsTrId("whisperfish-bug-report")
                onClicked: {
                    Qt.openUrlExternally("https://gitlab.com/whisperfish/whisperfish/issues")
                }
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                //: Visit the Wiki button, tapping links to the Whisperfish Wiki
                //% "Visit the Wiki"
                text: qsTrId("whisperfish-about-wiki-link")
                onClicked: {
                    Qt.openUrlExternally("https://gitlab.com/whisperfish/whisperfish/-/wikis/home")
                }
            }

            SectionHeader {
                //: Additional Copyright
                //% "Additional Copyright"
                text: qsTrId("whisperfish-extra-copyright")
            }

            Label {
                text: "libsignal-client by Signal"
                anchors.horizontalCenter: parent.horizontalCenter
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                width: (parent ? parent.width : Screen.width) - Theme.paddingLarge * 2
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: Text.AlignLeft
                x: Theme.paddingLarge
            }

            Label {
                text: "libsignal-service-rs by Ruben De Smet, Gabriel Féron, and Michael Bryan"
                anchors.horizontalCenter: parent.horizontalCenter
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                width: (parent ? parent.width : Screen.width) - Theme.paddingLarge * 2
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: Text.AlignLeft
                x: Theme.paddingLarge
            }
        }
    }
}
