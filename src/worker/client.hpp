#pragma once

#include <QtCore>

class ClientWorker : public QObject {
    Q_OBJECT

public:
    ClientWorker(QObject *parent = nullptr): QObject(parent) {
    }

signals:
    void messageReceived(int sid, int mid);
    void messageReceipt(int sid, int mid);
    void notifyMessage(int sid, QString source, QString message, bool isGroup);
    void promptResetPeerIdentity(QString source);
};
