#ifndef WHISPERFISH_PLUGIN_INFO
#define WHISPERFISH_PLUGIN_INFO

#include <TransferEngine-qt5/transferplugininfo.h>
#include <TransferEngine-qt5/transfermethodinfo.h>

// Display Name
#define APP_NAME "Whisperfish"
#define APP_ICON "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
#define PLUGIN_ID "WhisperfishSharePlugin"

#define DBUS_INTERFACE "be.rubdos.whisperfish.share"
#define DBUS_SERVICE "be.rubdos.whisperfish"
#define DBUS_PATH "/be/rubdos/whisperfish"

class TransferMethodInfo;

class WhisperfishPluginInfo: public TransferPluginInfo
{
public:
    WhisperfishPluginInfo();

    QList<TransferMethodInfo> info() const Q_DECL_OVERRIDE;

    void query() Q_DECL_OVERRIDE;
    bool ready() const Q_DECL_OVERRIDE;

private:
    QList<TransferMethodInfo> infoList;
    bool is_ready;
};

#endif
