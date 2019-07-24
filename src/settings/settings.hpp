#pragma once

#include <QtCore>

class Settings : public QObject {
    Q_OBJECT

public:
    Settings(QObject *parent = nullptr): QObject(parent) {
    }
};
