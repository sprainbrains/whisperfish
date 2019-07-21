# NOTICE:
#
# Application name defined in TARGET has a corresponding QML filename.
# If name defined in TARGET is changed, the following needs to be done
# to match new name:
#   - corresponding QML filename must be changed
#   - desktop icon filename must be changed
#   - desktop filename must be changed
#   - icon definition filename in desktop file must be changed
#   - translation filenames have to be changed

# The name of your application
NAME = whisperfish
PREFIX = harbour
TARGET = $${PREFIX}-$${NAME}

CONFIG += sailfishapp link_pkgconfig
PKGCONFIG += sailfishapp mlite5

QT += concurrent sql

LIBS += -ldl

isEmpty(VERSION) {
    VERSION = 0.6.0
    message("VERSION is unset, assuming $$VERSION")
}

DEFINES += APP_VERSION=\\\"$$VERSION\\\"
DEFINES += GIT_CURRENT_SHA1="\\\"$(shell git -C \""$$_PRO_FILE_PWD_"\" describe)\\\""

INCLUDEPATH += \
    src

CONFIG(debug, debug|release) {
    DEFINES += HARBOUR_DEBUG=1
}

SOURCES += \
    src/harbour-whisperfish.cpp

# HEADERS += \

OTHER_FILES += \
    qml/cover/CoverPage.qml \
    rpm/harbour-whisperfish.spec \
    rpm/harbour-whisperfish.yaml \
    icons/*.svg \
    README.rst \
    harbour-whisperfish-message.conf \
    harbour-whisperfish.desktop \
    rpm/harbour-whisperfish.changes \
    qml/harbour-whisperfish.qml \
    qml/components/*.qml \
    qml/cover/cover-image.png \
    qml/pages/img/*.png \
    qml/pages/img/*.svg \
    qml/pages/*.qml

# Icons
ICON_SIZES = 86 108 128 256
for(s, ICON_SIZES) {
    icon_target = icon$${s}
    icon_dir = icons/$${s}x$${s}
    $${icon_target}.files = $${icon_dir}/$${TARGET}.png
    $${icon_target}.path = /usr/share/icons/hicolor/$${s}x$${s}/apps
    INSTALLS += $${icon_target}
}
