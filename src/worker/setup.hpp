#pragma once

#include <QtCore>

class SetupWorker : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString phoneNumber READ phoneNumber NOTIFY phoneNumberChanged);
    Q_PROPERTY(QString identity READ identity NOTIFY identityChanged);
    Q_PROPERTY(bool registered READ registered);

public:
    SetupWorker(QObject *parent = nullptr);

    QString phoneNumber() const;
    QString identity() const;
    bool registered() const;

signals:
    void registrationSuccess();
    void setupComplete();
    void invalidPhoneNumber();
    void invalidDatastore();
    void clientFailed();

    void phoneNumberChanged();
    void identityChanged();
};
