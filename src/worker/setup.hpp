#pragma once

#include <QtCore>

class SetupWorker : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString phoneNumber READ phoneNumber NOTIFY phoneNumberChanged);
    Q_PROPERTY(QString identity READ identity NOTIFY identityChanged);

public:
    SetupWorker(QObject *parent = nullptr): QObject(parent) {
    }

    QString phoneNumber() const;
    QString identity() const;

signals:
    void registrationSuccess();
    void setupComplete();
    void invalidPhoneNumber();
    void invalidDatastore();
    void clientFailed();

    void phoneNumberChanged();
    void identityChanged();
};
