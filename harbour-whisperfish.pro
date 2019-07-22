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

QT += core network concurrent sql

LIBS += -ldl

isEmpty(VERSION) {
    VERSION = 0.6.0
    message("VERSION is unset, assuming $$VERSION")
}

DEFINES += APP_VERSION=\\\"$$VERSION\\\"
DEFINES += GIT_CURRENT_SHA1="\\\"$(shell (cd \"$$_PRO_FILE_PWD_\"; git describe))\\\""

CONFIG += sailfishapp_i18n \
    sailfishapp_i18n_idbased \
    sailfishapp_i18n_unfinished

TRANSLATIONS += \
    translations/harbour-whisperfish.ts \
    translations/harbour-whisperfish-de.ts \
    translations/harbour-whisperfish-es.ts \
    translations/harbour-whisperfish-fi.ts \
    translations/harbour-whisperfish-hu.ts \
    translations/harbour-whisperfish-nl.ts \
    translations/harbour-whisperfish-nl_BE.ts \
    translations/harbour-whisperfish-pl.ts \

INCLUDEPATH += \
    src \
    libsignal-protocol-c/src \

CONFIG(debug, debug|release) {
    DEFINES += HARBOUR_DEBUG=1
}

SOURCES += \
    src/harbour-whisperfish.cpp

HEADERS += \
    src/whisperfish.hpp \

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
    qml/pages/*.qml \
    translations/*.ts \
    libsignal-protocol-c \

libsignal.target = libsignal-build/src/libsignal-protocol-c.a
libsignal.commands = \
    mkdir -p libsignal-build/ ; \
    ( cd libsignal-build/ ; cmake -DCMAKE_BUILD_TYPE=Release "$$_PRO_FILE_PWD_/libsignal-protocol-c/") ; \ # holy shit this is hacky
    $(MAKE) -C libsignal-build ; \


QMAKE_EXTRA_TARGETS += libsignal
PRE_TARGETDEPS += libsignal-build/src/libsignal-protocol-c.a
LIBS += -Llibsignal-build/src/ -lsignal-protocol-c

# Icons
ICON_SIZES = 86 108 128 256
for(s, ICON_SIZES) {
    icon_target = icon$${s}
    icon_dir = icons/$${s}x$${s}
    $${icon_target}.files = $${icon_dir}/$${TARGET}.png
    $${icon_target}.path = /usr/share/icons/hicolor/$${s}x$${s}/apps
    INSTALLS += $${icon_target}
}
