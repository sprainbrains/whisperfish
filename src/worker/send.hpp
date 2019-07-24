#pragma once

#include <QtCore>

class SendWorker : public QObject {
    Q_OBJECT

public:
    SendWorker(QObject *parent = nullptr): QObject(parent) {
    }
};
