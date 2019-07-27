#pragma once

#include <QtCore>

class Prompt : public QObject {
    Q_OBJECT

public:
    Prompt(QObject *parent = nullptr): QObject(parent) {
    }

signals:
    void promptPhoneNumber();
    void promptVerificationCode();
    void promptPassword();
    void promptResetPeerIdentity(QString source);

    void receivePassword(QString password);

public slots:
    void password(const QString string);
};
