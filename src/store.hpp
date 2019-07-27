#pragma once

#include <QtCore>

class Prompt;

class Store : public QObject {
    Q_OBJECT

public:
    Store();

    void loadIdentity(Prompt *);

signals:
    void ready();
    void decryptionFailure();

private slots:
    void supplyPassword(const QString password);

private:
    bool derive_keys(const QString password);
    bool read_identity_keys();

    bool encrypted;
    QString storage_path;
    QByteArray identity_keys;
    QByteArray salt;
    QByteArray key_material;
};
