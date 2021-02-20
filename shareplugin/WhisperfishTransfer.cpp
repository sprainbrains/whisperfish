#include "WhisperfishPluginInfo.h"
#include "WhisperfishTransfer.h"

WhisperfishTransfer::WhisperfishTransfer(QObject *parent) : MediaTransferInterface(parent)
{ 
}

bool WhisperfishTransfer::cancelEnabled() const
{ 
	return false; 
}

QString WhisperfishTransfer::displayName() const
{ 
	return APP_NAME; 
}

bool WhisperfishTransfer::restartEnabled() const
{ 
	return false; 
}

QUrl WhisperfishTransfer::serviceIcon() const
{ 
	return QUrl::fromLocalFile(APP_ICON); 
}

void WhisperfishTransfer::cancel()
{
}

void WhisperfishTransfer::start()
{
}
