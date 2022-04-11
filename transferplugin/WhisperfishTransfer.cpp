#include "WhisperfishTransfer.h"
#include "mediaitem.h"

WhisperfishTransfer::WhisperfishTransfer(QObject *parent) : MediaTransferInterface(parent)
{
}

WhisperfishTransfer::~WhisperfishTransfer()
{
}

QString WhisperfishTransfer::displayName() const
{
    return "Whisperfish";
}

QUrl WhisperfishTransfer::serviceIcon() const
{
    // Url to the icon which should be shown in the transfer UI
    return QUrl("image://theme/icon-s-message");
}

bool WhisperfishTransfer::cancelEnabled() const
{
    // Return true if cancelling ongoing upload is supported
    // Return false if cancelling ongoing upload is not supported
    return false;
}

bool WhisperfishTransfer::restartEnabled() const
{
    // Return true, if restart is  supported.
    // Return false, if restart is not supported
    return false;
}


void WhisperfishTransfer::start()
{
    // This is called by the sharing framework to start sharing

    // TODO: Add your code here to start uploading
}

void WhisperfishTransfer::cancel()
{
    // This is called by the sharing framework to cancel on going transfer

    // TODO: Add your code here to cancel ongoing upload
}