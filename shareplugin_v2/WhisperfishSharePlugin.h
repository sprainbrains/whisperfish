#ifndef WHISPERFISH_SHARE_PLUGIN
#define WHISPERFISH_SHARE_PLUGIN

#include "sharingplugininterface.h"
#include <QObject>

class Q_DECL_EXPORT WhisperfishSharePlugin : public QObject, public SharingPluginInterface
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID "be.rubdos.whisperfish.share.plugin")
    Q_INTERFACES(SharingPluginInterface)

public:
    WhisperfishSharePlugin() {};
    ~WhisperfishSharePlugin() {};

    QString pluginId() const;
    SharingPluginInfo* infoObject();
};

#endif // WHISPERFISH_SHARE_PLUGIN
