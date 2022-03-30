TEMPLATE = lib
TARGET = $$qtLibraryTarget(whisperfishshareplugin)
CONFIG += plugin
DEPENDPATH += .

CONFIG += link_pkgconfig
PKGCONFIG += nemotransferengine-qt5

HEADERS += \
    WhisperfishPluginInfo.h \
    WhisperfishSharePlugin.h

SOURCES += \
    WhisperfishPluginInfo.cpp \
    WhisperfishSharePlugin.cpp

OTHER_FILES += \
    WhisperfishShare.qml

shareui.files = *.qml
shareui.path = /usr/share/nemo-transferengine/plugins/sharing

target.path = $$LIBDIR/nemo-transferengine/plugins/sharing
INSTALLS += target shareui
