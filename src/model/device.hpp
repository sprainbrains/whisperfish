#pragma once

#include <QtCore>

class DeviceModel : public QObject {
    Q_OBJECT

public:
    DeviceModel(QObject *parent = nullptr): QObject(parent) {
    }
};
