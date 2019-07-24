#pragma once

#include <QtCore>

class MessageModel : public QObject {
    Q_OBJECT

public:
    MessageModel(QObject *parent = nullptr): QObject(parent) {
    }
};
