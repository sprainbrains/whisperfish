#pragma once

#include <QtCore>

class ClientWorker : public QObject {
    Q_OBJECT

public:
    ClientWorker(QObject *parent = nullptr): QObject(parent) {
    }
};
