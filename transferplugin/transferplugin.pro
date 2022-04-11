TEMPLATE = lib
TARGET = $$qtLibraryTarget(whisperfishtransferplugin)
CONFIG += plugin
DEPENDPATH += .

CONFIG += link_pkgconfig
PKGCONFIG += nemotransferengine-qt5

HEADERS += \
    WhisperfishTransfer.h \
    WhisperfishTransferPlugin.h

SOURCES += \
    WhisperfishTransfer.cpp \
    WhisperfishTransferPlugin.cpp

target.path = $$LIBDIR/nemo-transferengine/plugins/transfer
INSTALLS += target
