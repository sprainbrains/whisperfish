#include "WhisperfishSharePlugin.h"
#include "WhisperfishPluginInfo.h"
#include <QtPlugin>

SharingPluginInfo* WhisperfishSharePlugin::infoObject()
{ 
    return new WhisperfishPluginInfo;
}

QString WhisperfishSharePlugin::pluginId() const
{ 
    return PLUGIN_ID;
}
