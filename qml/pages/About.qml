import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
	id: aboutpage
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
				source: "/usr/share/icons/hicolor/86x86/apps/harbour-whisperfish.png"
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

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
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
                    "Ruben De Smet (2019-2021)\n" +
                    "Markus Törnqvist (2019-2021)\n" +
                    "Mirian Margiani (2021)\n" +
                    "Gabriel Margiani (2021)\n" +
                    "Thomas Michel (2021)\n" +
                    "License: AGPLv3 & GPLv3"
                }
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
                    Qt.openUrlExternally("https://gitlab.com/rubdos/whisperfish")
                }
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                //: Report a Bug
                //% "Report a Bug"
                text: qsTrId("whisperfish-bug-report")
                onClicked: {
                    Qt.openUrlExternally("https://gitlab.com/rubdos/whisperfish/issues")
                }
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                //: Visit the Wiki button, tapping links to the Whisperfish Wiki
                //% "Visit the Wiki"
                text: qsTrId("whisperfish-about-wiki-link")
                onClicked: {
                    Qt.openUrlExternally("https://gitlab.com/rubdos/whisperfish/-/wikis/home")
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
