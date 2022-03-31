#include "WhisperfishTransferPlugin.h"
#include "WhisperfishTransfer.h"
#include "WhisperfishPluginInfo.h"

TransferPluginInfo* WhisperfishTransferPlugin::infoObject()
{ 
    return new WhisperfishPluginInfo;
}

MediaTransferInterface* WhisperfishTransferPlugin::transferObject()
{ 
    return new WhisperfishTransfer;
}

QString WhisperfishTransferPlugin::pluginId() const
{ 
    return PLUGIN_ID;
}

bool WhisperfishTransferPlugin::enabled() const
{ 
    return true;
}

