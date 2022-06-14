TEMPLATE = lib
TARGET = $$qtLibraryTarget(whisperfishshareplugin)
CONFIG += plugin
DEPENDPATH += .

CONFIG += link_pkgconfig
PKGCONFIG += nemotransferengine-qt5

HEADERS += \
    WhisperfishPluginInfo.h \
    WhisperfishTransfer.h \
    WhisperfishTransferPlugin.h

SOURCES += \
    WhisperfishPluginInfo.cpp \
    WhisperfishTransfer.cpp \
    WhisperfishTransferPlugin.cpp

OTHER_FILES += \
    WhisperfishShare.qml

shareui.files = *.qml
shareui.path = /usr/share/nemo-transferengine/plugins

target.path = $$LIBDIR/nemo-transferengine/plugins
INSTALLS += target shareui
