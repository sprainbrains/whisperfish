#include "worker/client.hpp"

#include <QtWebSockets/QtWebSockets>
#include <QtCore/QList>

const QString websocketPath = "/v1/websocket/";
const QString rootPEM = R"(
-----BEGIN CERTIFICATE-----
MIID7zCCAtegAwIBAgIJAIm6LatK5PNiMA0GCSqGSIb3DQEBBQUAMIGNMQswCQYDVQQGEwJVUzET
MBEGA1UECAwKQ2FsaWZvcm5pYTEWMBQGA1UEBwwNU2FuIEZyYW5jaXNjbzEdMBsGA1UECgwUT3Bl
biBXaGlzcGVyIFN5c3RlbXMxHTAbBgNVBAsMFE9wZW4gV2hpc3BlciBTeXN0ZW1zMRMwEQYDVQQD
DApUZXh0U2VjdXJlMB4XDTEzMDMyNTIyMTgzNVoXDTIzMDMyMzIyMTgzNVowgY0xCzAJBgNVBAYT
AlVTMRMwEQYDVQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1TYW4gRnJhbmNpc2NvMR0wGwYDVQQK
DBRPcGVuIFdoaXNwZXIgU3lzdGVtczEdMBsGA1UECwwUT3BlbiBXaGlzcGVyIFN5c3RlbXMxEzAR
BgNVBAMMClRleHRTZWN1cmUwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDBSWBpOCBD
F0i4q2d4jAXkSXUGpbeWugVPQCjaL6qD9QDOxeW1afvfPo863i6Crq1KDxHpB36EwzVcjwLkFTIM
eo7t9s1FQolAt3mErV2U0vie6Ves+yj6grSfxwIDAcdsKmI0a1SQCZlr3Q1tcHAkAKFRxYNawADy
ps5B+Zmqcgf653TXS5/0IPPQLocLn8GWLwOYNnYfBvILKDMItmZTtEbucdigxEA9mfIvvHADEbte
LtVgwBm9R5vVvtwrD6CCxI3pgH7EH7kMP0Od93wLisvn1yhHY7FuYlrkYqdkMvWUrKoASVw4jb69
vaeJCUdU+HCoXOSP1PQcL6WenNCHAgMBAAGjUDBOMB0GA1UdDgQWBBQBixjxP/s5GURuhYa+lGUy
pzI8kDAfBgNVHSMEGDAWgBQBixjxP/s5GURuhYa+lGUypzI8kDAMBgNVHRMEBTADAQH/MA0GCSqG
SIb3DQEBBQUAA4IBAQB+Hr4hC56m0LvJAu1RK6NuPDbTMEN7/jMojFHxH4P3XPFfupjR+bkDq0pP
OU6JjIxnrD1XD/EVmTTaTVY5iOheyv7UzJOefb2pLOc9qsuvI4fnaESh9bhzln+LXxtCrRPGhkxA
1IMIo3J/s2WF/KVYZyciu6b4ubJ91XPAuBNZwImug7/srWvbpk0hq6A6z140WTVSKtJG7EP41kJe
/oF4usY5J7LPkxK3LWzMJnb5EIJDmRvyH8pyRwWg6Qm6qiGFaI4nL8QU4La1x2en4DGXRaLMPRwj
ELNgQPodR38zoCMuA8gHZfZYYoZ7D7Q1wNUiVHcxuFrEeBaYJbLErwLV
-----END CERTIFICATE-----
)";

ClientWorker::ClientWorker(QObject *parent):
    QObject(parent)
{
    // XXX: un-hardcode
    connect(&wss, &QWebSocket::connected, this, &ClientWorker::onConnected);
    connect(&wss, &QWebSocket::disconnected, this, &ClientWorker::onDisconnect);
    connect(&wss, &QWebSocket::sslErrors,
            this, &ClientWorker::onSslErrors);
    // These static_casts can be replaced with QOverload for C++14 and Qt>5.7
    connect(&wss, static_cast<void (QWebSocket::*)(QAbstractSocket::SocketError)>(&QWebSocket::error), this, &ClientWorker::onError);
    connect(&wss, &QWebSocket::textMessageReceived, this, &ClientWorker::onTextMessageReceived);
    connect(&wss, &QWebSocket::binaryMessageReceived, this, &ClientWorker::onBinaryMessageReceived);


    // Add Signal's custom root certificate
    // TODO: set TLS level to highest supported by server.
    auto config = wss.sslConfiguration();
    QList<QSslCertificate> certs;
    certs.append(QSslCertificate::fromData(rootPEM.toUtf8()));
    config.setCaCertificates(certs);
    wss.setSslConfiguration(config);

    open();
}

void ClientWorker::open() {
    qInfo() << "Starting websocket";
    wss.open(QUrl("wss://textsecure-service.whispersystems.org" + websocketPath));
}

void ClientWorker::onError(QAbstractSocket::SocketError error) {
    qWarning() << "websocket error" << error;

    switch(error) {
    case QAbstractSocket::RemoteHostClosedError:
        // reconnect
        open();
        break;
    default:
        qWarning() << "Not handling error";
    }
}

void ClientWorker::onConnected() {
    qInfo() << "connected to signal";
}

void ClientWorker::onTextMessageReceived(const QString message) {
    qDebug() << "TextMessage received:" << message;
}

void ClientWorker::onBinaryMessageReceived(const QByteArray &message) {
    qDebug() << "BinaryMessage received of" << message.count() << "bytes";
}

void ClientWorker::onDisconnect() {
    qDebug() << "WebSocket disconnected because" << wss.closeReason();
}

void ClientWorker::onSslErrors(const QList<QSslError> errors) {
    Q_UNUSED(errors);

    qWarning() << "SSL errors!!";
    // XXX: graceful!
    wss.abort();
}
