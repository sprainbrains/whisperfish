#pragma once

#include <QtCore>

class SendWorker : public QObject {
    Q_OBJECT

public:
    SendWorker(QObject *parent = nullptr): QObject(parent) {
    }

signals:
    void sendMessage(int sid);
    void messageSent(int sid, int mid, QString message);
    void promptResetPeerIdentity(QString source);
};
