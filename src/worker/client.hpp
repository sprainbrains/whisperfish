#pragma once

#include <QtCore>
#include <QtWebSockets/QtWebSockets>
#include <QtNetwork>

class ClientWorker : public QObject {
    Q_OBJECT

public:
    ClientWorker(QObject *parent = nullptr);

signals:
    void messageReceived(int sid, int mid);
    void messageReceipt(int sid, int mid);
    void notifyMessage(int sid, QString source, QString message, bool isGroup);
    void promptResetPeerIdentity(QString source);

private slots:
    void onConnected();
    void onTextMessageReceived(const QString message);
    void onSslErrors(const QList<QSslError>);
    void onError(QAbstractSocket::SocketError);

private:
    QWebSocket wss;
};
