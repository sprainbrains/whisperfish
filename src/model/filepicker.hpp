#pragma once

#include <QtCore>

class FilePicker : public QObject {
    Q_OBJECT

public:
    FilePicker(QObject *parent = nullptr): QObject(parent) {
    }
};
