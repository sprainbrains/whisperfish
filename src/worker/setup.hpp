#pragma once

#include <QtCore>

class SetupWorker : public QObject {
    Q_OBJECT

public:
    SetupWorker(QObject *parent = nullptr): QObject(parent) {
    }

signals:
    void registrationSuccess();
    void setupComplete();
    void invalidPhoneNumber();
    void invalidDatastore();
    void clientFailed();
};
