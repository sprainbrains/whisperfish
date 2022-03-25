#ifndef WHISPERFISH_PLUGIN_INFO
#define WHISPERFISH_PLUGIN_INFO

#include "sharingplugininfo.h"

// Display Name
#define APP_NAME "Whisperfish"
#define APP_ICON "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
#define PLUGIN_ID "WhisperfishSharePlugin"

#define DBUS_INTERFACE "be.rubdos.whisperfish.share"
#define DBUS_SERVICE "be.rubdos.whisperfish"
#define DBUS_PATH "/be/rubdos/whisperfish"

class WhisperfishPluginInfo: public SharingPluginInfo
{
	public:
		WhisperfishPluginInfo();
		~WhisperfishPluginInfo();

		QList<SharingMethodInfo> info() const;
		void query();
		bool ready() const;

	private:
		QList<SharingMethodInfo> infoList;
		bool is_ready;
};

#endif
