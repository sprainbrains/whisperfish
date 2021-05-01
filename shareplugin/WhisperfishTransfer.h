#ifndef WHISPERFISH_MEDIA_TRANSFER
#define WHISPERFISH_MEDIA_TRANSFER

#include <QObject>

#include <TransferEngine-qt5/mediatransferinterface.h>
#include <TransferEngine-qt5/mediaitem.h>

#include <QtDBus/QtDBus>

class WhisperfishTransfer : public MediaTransferInterface
{
	Q_OBJECT

	public:
		explicit WhisperfishTransfer(QObject *parent = nullptr);

		bool cancelEnabled() const;
		QString displayName() const;
		bool restartEnabled() const;
		QUrl serviceIcon() const;

	public slots:
		void	cancel();
		void	start();
};

#endif
