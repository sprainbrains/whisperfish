#include "WhisperfishTransferPlugin.h"
#include "WhisperfishTransfer.h"
#include <QtPlugin>

MediaTransferInterface* WhisperfishTransferPlugin::transferObject()
{ 
	return new WhisperfishTransfer; 
}

QString WhisperfishTransferPlugin::pluginId() const
{ 
	return PLUGIN_ID; 
}