#pragma once

#include <QtCore>

class SetupWorker : public QObject {
    Q_OBJECT

public:
    SetupWorker(QObject *parent = nullptr): QObject(parent) {
    }
};
