#pragma once

#include <QtCore>

class ContactModel : public QObject {
    Q_OBJECT

public:
    ContactModel(QObject *parent = nullptr): QObject(parent) {
    }
};
