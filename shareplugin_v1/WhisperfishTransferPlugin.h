#ifndef WHISPERFISH_TRANSFER_PLUGIN
#define WHISPERFISH_TRANSFER_PLUGIN

#include <TransferEngine-qt5/transferplugininterface.h>
#include <TransferEngine-qt5/transferplugininfo.h>
#include <TransferEngine-qt5/transfermethodinfo.h>
#include <TransferEngine-qt5/mediatransferinterface.h>

class WhisperfishTransferPlugin : public QObject, public TransferPluginInterface
{
    Q_OBJECT
	Q_PLUGIN_METADATA(IID "be.rubdos.whisperfish.transfer.plugin")
	Q_INTERFACES(TransferPluginInterface)

	public:

		WhisperfishTransferPlugin() {};

		QString pluginId() const;
		bool enabled() const;

		TransferPluginInfo* infoObject();
		MediaTransferInterface* transferObject();
};

#endif
