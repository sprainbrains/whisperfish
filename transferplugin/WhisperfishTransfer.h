#ifndef WHISPERFISH_TRANSFER
#define WHISPERFISH_TRANSFER

#include "mediatransferinterface.h"

// Display Name
#define APP_NAME "Whisperfish"
#define APP_ICON "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
#define PLUGIN_ID "WhisperfishTransferPlugin"

class WhisperfishTransfer : public MediaTransferInterface
{
    Q_OBJECT

public:
    WhisperfishTransfer(QObject *parent = nullptr);
    ~WhisperfishTransfer();

    bool cancelEnabled() const;
    QString displayName() const;
    bool restartEnabled() const;
    QUrl serviceIcon() const;

public slots:
    void    cancel();
    void    start();
};

#endif // WHISPERFISH_TRANSFER
