#include "WhisperfishPluginInfo.h"

WhisperfishPluginInfo::WhisperfishPluginInfo(): 
	is_ready(false) 
{}

QList<TransferMethodInfo> WhisperfishPluginInfo::info() const
{ 
	return infoList; 
}

bool WhisperfishPluginInfo::ready() const
{ 
	return is_ready; 
}

void WhisperfishPluginInfo::query()
{
    TransferMethodInfo info;
    //QFileInfo png(QRSHARE_ICON_PNG);

    info.displayName = QLatin1String(APP_NAME);
    info.methodId = QLatin1String(PLUGIN_ID);
	info.accountIcon = QLatin1String(APP_ICON);
	info.shareUIPath = QLatin1String(
			"/usr/share/nemo-transferengine/plugins/WhisperfishShare.qml");

	// TODO: Check if we should be more specific here - this is what depecher sets.
    info.capabilitities << QLatin1String("application/*")
                 << QLatin1String("image/*")
                 << QLatin1String("audio/*")
                 << QLatin1String("video/*")
                 << QLatin1String("text/plain")
                 << QLatin1String("text/x-url")
                 << QLatin1String("text/vcard");

    infoList.clear();
    infoList << info;

    is_ready = true;
    Q_EMIT infoReady();
}
